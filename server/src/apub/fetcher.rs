use activitystreams::object::Note;
use actix_web::client::Client;
use diesel::{result::Error::NotFound, PgConnection};
use log::debug;
use serde::Deserialize;
use std::{
  fmt::Debug,
  time::{Duration, Instant},
};
use url::Url;

use crate::{
  api::site::SearchResponse,
  db::{
    comment::{Comment, CommentForm},
    comment_view::CommentView,
    community::{Community, CommunityForm, CommunityModerator, CommunityModeratorForm},
    community_view::CommunityView,
    post::{Post, PostForm},
    post_view::PostView,
    user::{UserForm, User_},
    Crud, Joinable, SearchType,
  },
  naive_now,
  routes::nodeinfo::{NodeInfo, NodeInfoWellKnown},
  DbPool, LemmyError, RecvError, SendError,
};

use crate::{
  apub::{
    get_apub_protocol_string, is_apub_id_valid, FromApub, GroupExt, PageExt, PersonExt,
    APUB_JSON_CONTENT_TYPE,
  },
  db::user_view::UserView,
};
use chrono::NaiveDateTime;

static ACTOR_REFETCH_INTERVAL_SECONDS: i64 = 24 * 60 * 60;

// Fetch nodeinfo metadata from a remote instance.
async fn _fetch_node_info(client: &Client, domain: &str) -> Result<NodeInfo, LemmyError> {
  let well_known_uri = Url::parse(&format!(
    "{}://{}/.well-known/nodeinfo",
    get_apub_protocol_string(),
    domain
  ))?;

  let well_known = fetch_remote_object::<NodeInfoWellKnown>(client, &well_known_uri).await?;
  let nodeinfo = fetch_remote_object::<NodeInfo>(client, &well_known.links.href).await?;

  Ok(nodeinfo)
}

async fn do_fetch<Response>(
  client: &Client,
  url: &Url,
  timeout: Duration,
) -> Result<Response, LemmyError>
where
  Response: for<'de> Deserialize<'de>,
{
  let json = client
    .get(url.as_str())
    .header("Accept", APUB_JSON_CONTENT_TYPE)
    .timeout(timeout)
    .send()
    .await
    .map_err(|e| {
      debug!("Send error, {}", e);
      SendError(e.to_string())
    })?
    .json()
    .await
    .map_err(|e| {
      debug!("Receive error, {}", e);
      RecvError(e.to_string())
    })?;

  Ok(json)
}

/// Fetch any type of ActivityPub object, handling things like HTTP headers, deserialisation,
/// timeouts etc.
pub async fn fetch_remote_object<Response>(
  client: &Client,
  url: &Url,
) -> Result<Response, LemmyError>
where
  Response: for<'de> Deserialize<'de>,
{
  if !is_apub_id_valid(&url) {
    return Err(format_err!("Activitypub uri invalid or blocked: {}", url).into());
  }

  let mut err = format_err!("This should never happen").into();

  let timeout = Duration::from_secs(60);
  let start = Instant::now();

  // retry up to 2 additional times as long as we haven't hit the timeout
  for i in 0..3 {
    match do_fetch(client, url, timeout).await {
      Ok(json) => return Ok(json),
      Err(e) => err = e,
    }

    debug!("Failed request {}", i);

    let elapsed = Instant::now();
    if elapsed - start > timeout {
      debug!("timout");
      break;
    }
  }

  Err(err)
}

/// The types of ActivityPub objects that can be fetched directly by searching for their ID.
#[serde(untagged)]
#[derive(serde::Deserialize, Debug)]
pub enum SearchAcceptedObjects {
  Person(Box<PersonExt>),
  Group(Box<GroupExt>),
  Page(Box<PageExt>),
  Comment(Box<Note>),
}

/// Attempt to parse the query as URL, and fetch an ActivityPub object from it.
///
/// Some working examples for use with the docker/federation/ setup:
/// http://lemmy_alpha:8540/c/main, or !main@lemmy_alpha:8540
/// http://lemmy_alpha:8540/u/lemmy_alpha, or @lemmy_alpha@lemmy_alpha:8540
/// http://lemmy_alpha:8540/post/3
/// http://lemmy_alpha:8540/comment/2
pub async fn search_by_apub_id(
  query: &str,
  client: &Client,
  pool: DbPool,
) -> Result<SearchResponse, LemmyError> {
  // Parse the shorthand query url
  let query_url = if query.contains('@') {
    debug!("{}", query);
    let split = query.split('@').collect::<Vec<&str>>();

    // User type will look like ['', username, instance]
    // Community will look like [!community, instance]
    let (name, instance) = if split.len() == 3 {
      (format!("/u/{}", split[1]), split[2])
    } else if split.len() == 2 {
      if split[0].contains('!') {
        let split2 = split[0].split('!').collect::<Vec<&str>>();
        (format!("/c/{}", split2[1]), split[1])
      } else {
        return Err(format_err!("Invalid search query: {}", query).into());
      }
    } else {
      return Err(format_err!("Invalid search query: {}", query).into());
    };

    let url = format!("{}://{}{}", get_apub_protocol_string(), instance, name);
    Url::parse(&url)?
  } else {
    Url::parse(&query)?
  };

  let mut response = SearchResponse {
    type_: SearchType::All.to_string(),
    comments: vec![],
    posts: vec![],
    communities: vec![],
    users: vec![],
  };

  let response = match fetch_remote_object::<SearchAcceptedObjects>(client, &query_url).await? {
    SearchAcceptedObjects::Person(p) => {
      let user_uri = p.inner.object_props.get_id().unwrap().to_string();

      let user = get_or_fetch_and_upsert_remote_user(&user_uri, client, pool.clone()).await?;

      response.users = vec![unblock!(pool, conn, UserView::read(&conn, user.id)?)];

      response
    }
    SearchAcceptedObjects::Group(g) => {
      let community_uri = g.inner.object_props.get_id().unwrap().to_string();

      let community =
        get_or_fetch_and_upsert_remote_community(&community_uri, client, pool.clone()).await?;

      // TODO Maybe at some point in the future, fetch all the history of a community
      // fetch_community_outbox(&c, conn)?;
      response.communities = vec![unblock!(
        pool,
        conn,
        CommunityView::read(&conn, community.id, None)?
      )];

      response
    }
    SearchAcceptedObjects::Page(p) => {
      let post_form = PostForm::from_apub(&p, client, pool.clone()).await?;

      let p: Post = unblock!(pool, conn, upsert_post(&post_form, &conn)?);
      response.posts = vec![unblock!(pool, conn, PostView::read(&conn, p.id, None)?)];

      response
    }
    SearchAcceptedObjects::Comment(c) => {
      let post_url = c
        .object_props
        .get_many_in_reply_to_xsd_any_uris()
        .unwrap()
        .next()
        .unwrap()
        .to_string();

      // TODO: also fetch parent comments if any
      let post = fetch_remote_object(client, &Url::parse(&post_url)?).await?;
      let post_form = PostForm::from_apub(&post, client, pool.clone()).await?;
      let comment_form = CommentForm::from_apub(&c, client, pool.clone()).await?;

      let _: Post = unblock!(pool, conn, upsert_post(&post_form, &conn)?);
      let c: Comment = unblock!(pool, conn, upsert_comment(&comment_form, &conn)?);
      response.comments = vec![unblock!(pool, conn, CommentView::read(&conn, c.id, None)?)];

      response
    }
  };

  Ok(response)
}

/// Check if a remote user exists, create if not found, if its too old update it.Fetch a user, insert/update it in the database and return the user.
pub async fn get_or_fetch_and_upsert_remote_user(
  apub_id: &str,
  client: &Client,
  pool: DbPool,
) -> Result<User_, LemmyError> {
  let apub_id_owned = apub_id.to_owned();
  let user: Result<User_, _> =
    unblock!(pool, conn, User_::read_from_actor_id(&conn, &apub_id_owned));

  match user {
    // If its older than a day, re-fetch it
    Ok(u) if !u.local && should_refetch_actor(u.last_refreshed_at) => {
      debug!("Fetching and updating from remote user: {}", apub_id);
      let person = fetch_remote_object::<PersonExt>(client, &Url::parse(apub_id)?).await?;

      let mut uf = UserForm::from_apub(&person, client, pool.clone()).await?;
      uf.last_refreshed_at = Some(naive_now());
      let user: User_ = unblock!(pool, conn, User_::update(&conn, u.id, &uf)?);

      Ok(user)
    }
    Ok(u) => Ok(u),
    Err(NotFound {}) => {
      debug!("Fetching and creating remote user: {}", apub_id);
      let person = fetch_remote_object::<PersonExt>(client, &Url::parse(apub_id)?).await?;

      let uf = UserForm::from_apub(&person, client, pool.clone()).await?;
      let user: User_ = unblock!(pool, conn, User_::create(&conn, &uf)?);

      Ok(user)
    }
    Err(e) => Err(e.into()),
  }
}

/// Determines when a remote actor should be refetched from its instance. In release builds, this is
/// ACTOR_REFETCH_INTERVAL_SECONDS after the last refetch, in debug builds always.
///
/// TODO it won't pick up new avatars, summaries etc until a day after.
/// Actors need an "update" activity pushed to other servers to fix this.
fn should_refetch_actor(last_refreshed: NaiveDateTime) -> bool {
  if cfg!(debug_assertions) {
    true
  } else {
    let update_interval = chrono::Duration::seconds(ACTOR_REFETCH_INTERVAL_SECONDS);
    last_refreshed.lt(&(naive_now() - update_interval))
  }
}

/// Check if a remote community exists, create if not found, if its too old update it.Fetch a community, insert/update it in the database and return the community.
pub async fn get_or_fetch_and_upsert_remote_community(
  apub_id: &str,
  client: &Client,
  pool: DbPool,
) -> Result<Community, LemmyError> {
  let apub_id_owned = apub_id.to_owned();
  let community: Result<Community, _> = unblock!(
    pool,
    conn,
    Community::read_from_actor_id(&conn, &apub_id_owned)
  );

  match community {
    Ok(c) if !c.local && should_refetch_actor(c.last_refreshed_at) => {
      debug!("Fetching and updating from remote community: {}", apub_id);
      let group = fetch_remote_object::<GroupExt>(client, &Url::parse(apub_id)?).await?;

      let mut cf = CommunityForm::from_apub(&group, client, pool.clone()).await?;
      cf.last_refreshed_at = Some(naive_now());
      let community: Community = unblock!(pool, conn, Community::update(&conn, c.id, &cf)?);

      Ok(community)
    }
    Ok(c) => Ok(c),
    Err(NotFound {}) => {
      debug!("Fetching and creating remote community: {}", apub_id);
      let group = fetch_remote_object::<GroupExt>(client, &Url::parse(apub_id)?).await?;

      let cf = CommunityForm::from_apub(&group, client, pool.clone()).await?;
      let community: Community = unblock!(pool, conn, Community::create(&conn, &cf)?);

      // Also add the community moderators too
      let creator_and_moderator_uris = group
        .inner
        .object_props
        .get_many_attributed_to_xsd_any_uris()
        .unwrap();

      let mut creator_and_moderators = Vec::new();

      for uri in creator_and_moderator_uris {
        let c_or_m =
          get_or_fetch_and_upsert_remote_user(uri.as_str(), client, pool.clone()).await?;

        creator_and_moderators.push(c_or_m);
      }

      let community_id = community.id;
      let _: () = unblock!(pool, conn, {
        for mod_ in creator_and_moderators {
          let community_moderator_form = CommunityModeratorForm {
            community_id,
            user_id: mod_.id,
          };

          CommunityModerator::join(&conn, &community_moderator_form)?;
        }
      });

      Ok(community)
    }
    Err(e) => Err(e.into()),
  }
}

fn upsert_post(post_form: &PostForm, conn: &PgConnection) -> Result<Post, LemmyError> {
  let existing = Post::read_from_apub_id(conn, &post_form.ap_id);
  match existing {
    Err(NotFound {}) => Ok(Post::create(conn, &post_form)?),
    Ok(p) => Ok(Post::update(conn, p.id, &post_form)?),
    Err(e) => Err(e.into()),
  }
}

pub async fn get_or_fetch_and_insert_remote_post(
  post_ap_id: &str,
  client: &Client,
  pool: DbPool,
) -> Result<Post, LemmyError> {
  let post_ap_id_owned = post_ap_id.to_owned();
  let post: Result<_, _> = unblock!(
    pool,
    conn,
    Post::read_from_apub_id(&conn, &post_ap_id_owned)
  );

  match post {
    Ok(p) => Ok(p),
    Err(NotFound {}) => {
      debug!("Fetching and creating remote post: {}", post_ap_id);
      let post = fetch_remote_object::<PageExt>(client, &Url::parse(post_ap_id)?).await?;
      let post_form = PostForm::from_apub(&post, client, pool.clone()).await?;

      let post: Post = unblock!(pool, conn, Post::create(&conn, &post_form)?);

      Ok(post)
    }
    Err(e) => Err(e.into()),
  }
}

fn upsert_comment(comment_form: &CommentForm, conn: &PgConnection) -> Result<Comment, LemmyError> {
  let existing = Comment::read_from_apub_id(conn, &comment_form.ap_id);
  match existing {
    Err(NotFound {}) => Ok(Comment::create(conn, &comment_form)?),
    Ok(p) => Ok(Comment::update(conn, p.id, &comment_form)?),
    Err(e) => Err(e.into()),
  }
}

pub async fn get_or_fetch_and_insert_remote_comment(
  comment_ap_id: &str,
  client: &Client,
  pool: DbPool,
) -> Result<Comment, LemmyError> {
  let comment_ap_id_owned = comment_ap_id.to_owned();
  let comment: Result<_, _> = unblock!(
    pool,
    conn,
    Comment::read_from_apub_id(&conn, &comment_ap_id_owned)
  );

  match comment {
    Ok(p) => Ok(p),
    Err(NotFound {}) => {
      debug!(
        "Fetching and creating remote comment and its parents: {}",
        comment_ap_id
      );
      let comment = fetch_remote_object::<Note>(client, &Url::parse(comment_ap_id)?).await?;
      let comment_form = CommentForm::from_apub(&comment, client, pool.clone()).await?;

      let comment: Comment = unblock!(pool, conn, Comment::create(&conn, &comment_form)?);

      Ok(comment)
    }
    Err(e) => Err(e.into()),
  }
}

// TODO It should not be fetching data from a community outbox.
// All posts, comments, comment likes, etc should be posts to our community_inbox
// The only data we should be periodically fetching (if it hasn't been fetched in the last day
// maybe), is community and user actors
// and user actors
// Fetch all posts in the outbox of the given user, and insert them into the database.
// fn fetch_community_outbox(community: &Community, conn: &PgConnection) -> Result<Vec<Post>, LemmyError> {
//   let outbox_url = Url::parse(&community.get_outbox_url())?;
//   let outbox = fetch_remote_object::<OrderedCollection>(&outbox_url)?;
//   let items = outbox.collection_props.get_many_items_base_boxes();

//   Ok(
//     items
//       .unwrap()
//       .map(|obox: &BaseBox| -> Result<PostForm, LemmyError> {
//         let page = obox.clone().to_concrete::<Page>()?;
//         PostForm::from_page(&page, conn)
//       })
//       .map(|pf| upsert_post(&pf?, conn))
//       .collect::<Result<Vec<Post>, LemmyError>>()?,
//   )
// }
