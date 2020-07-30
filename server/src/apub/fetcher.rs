use crate::{
  api::site::SearchResponse,
  apub::{
    is_apub_id_valid,
    ActorType,
    FromApub,
    GroupExt,
    PageExt,
    PersonExt,
    APUB_JSON_CONTENT_TYPE,
  },
  blocking,
  request::{retry, RecvError},
  routes::nodeinfo::{NodeInfo, NodeInfoWellKnown},
  DbPool,
  LemmyError,
};
use activitystreams_new::{base::BaseExt, collection::OrderedCollection, object::Note, prelude::*};
use actix_web::client::Client;
use chrono::NaiveDateTime;
use diesel::{result::Error::NotFound, PgConnection};
use lemmy_db::{
  comment::{Comment, CommentForm},
  comment_view::CommentView,
  community::{Community, CommunityForm, CommunityModerator, CommunityModeratorForm},
  community_view::CommunityView,
  naive_now,
  post::{Post, PostForm},
  post_view::PostView,
  user::{UserForm, User_},
  user_view::UserView,
  Crud,
  Joinable,
  SearchType,
};
use lemmy_utils::get_apub_protocol_string;
use log::debug;
use serde::Deserialize;
use std::{fmt::Debug, time::Duration};
use url::Url;

static ACTOR_REFETCH_INTERVAL_SECONDS: i64 = 24 * 60 * 60;
static ACTOR_REFETCH_INTERVAL_SECONDS_DEBUG: i64 = 10;

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

  let timeout = Duration::from_secs(60);

  let json = retry(|| {
    client
      .get(url.as_str())
      .header("Accept", APUB_JSON_CONTENT_TYPE)
      .timeout(timeout)
      .send()
  })
  .await?
  .json()
  .await
  .map_err(|e| {
    debug!("Receive error, {}", e);
    RecvError(e.to_string())
  })?;

  Ok(json)
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
  pool: &DbPool,
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

  let domain = query_url.domain().unwrap();
  let response = match fetch_remote_object::<SearchAcceptedObjects>(client, &query_url).await? {
    SearchAcceptedObjects::Person(p) => {
      let user_uri = p.inner.id(domain)?.unwrap();

      let user = get_or_fetch_and_upsert_user(&user_uri, client, pool).await?;

      response.users = vec![blocking(pool, move |conn| UserView::read(conn, user.id)).await??];

      response
    }
    SearchAcceptedObjects::Group(g) => {
      let community_uri = g.inner.id(domain)?.unwrap();

      let community = get_or_fetch_and_upsert_community(community_uri, client, pool).await?;

      // TODO Maybe at some point in the future, fetch all the history of a community
      // fetch_community_outbox(&c, conn)?;
      response.communities = vec![
        blocking(pool, move |conn| {
          CommunityView::read(conn, community.id, None)
        })
        .await??,
      ];

      response
    }
    SearchAcceptedObjects::Page(p) => {
      let post_form = PostForm::from_apub(&p, client, pool).await?;

      let p = blocking(pool, move |conn| upsert_post(&post_form, conn)).await??;
      response.posts = vec![blocking(pool, move |conn| PostView::read(conn, p.id, None)).await??];

      response
    }
    SearchAcceptedObjects::Comment(c) => {
      let post_url = c.in_reply_to().as_ref().unwrap().as_many().unwrap();

      // TODO: also fetch parent comments if any
      let x = post_url.first().unwrap().as_xsd_any_uri().unwrap();
      let post = fetch_remote_object(client, x).await?;
      let post_form = PostForm::from_apub(&post, client, pool).await?;
      let comment_form = CommentForm::from_apub(&c, client, pool).await?;

      blocking(pool, move |conn| upsert_post(&post_form, conn)).await??;
      let c = blocking(pool, move |conn| upsert_comment(&comment_form, conn)).await??;
      response.comments =
        vec![blocking(pool, move |conn| CommentView::read(conn, c.id, None)).await??];

      response
    }
  };

  Ok(response)
}

pub async fn get_or_fetch_and_upsert_actor(
  apub_id: &Url,
  client: &Client,
  pool: &DbPool,
) -> Result<Box<dyn ActorType>, LemmyError> {
  let user = get_or_fetch_and_upsert_user(apub_id, client, pool).await;
  let actor: Box<dyn ActorType> = match user {
    Ok(u) => Box::new(u),
    Err(_) => Box::new(get_or_fetch_and_upsert_community(apub_id, client, pool).await?),
  };
  Ok(actor)
}

/// Check if a remote user exists, create if not found, if its too old update it.Fetch a user, insert/update it in the database and return the user.
pub async fn get_or_fetch_and_upsert_user(
  apub_id: &Url,
  client: &Client,
  pool: &DbPool,
) -> Result<User_, LemmyError> {
  let apub_id_owned = apub_id.to_owned();
  let user = blocking(pool, move |conn| {
    User_::read_from_actor_id(conn, apub_id_owned.as_str())
  })
  .await?;

  match user {
    // If its older than a day, re-fetch it
    Ok(u) if !u.local && should_refetch_actor(u.last_refreshed_at) => {
      debug!("Fetching and updating from remote user: {}", apub_id);
      let person = fetch_remote_object::<PersonExt>(client, apub_id).await?;

      let mut uf = UserForm::from_apub(&person, client, pool).await?;
      uf.last_refreshed_at = Some(naive_now());
      let user = blocking(pool, move |conn| User_::update(conn, u.id, &uf)).await??;

      Ok(user)
    }
    Ok(u) => Ok(u),
    Err(NotFound {}) => {
      debug!("Fetching and creating remote user: {}", apub_id);
      let person = fetch_remote_object::<PersonExt>(client, apub_id).await?;

      let uf = UserForm::from_apub(&person, client, pool).await?;
      let user = blocking(pool, move |conn| User_::create(conn, &uf)).await??;

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
  let update_interval = if cfg!(debug_assertions) {
    // avoid infinite loop when fetching community outbox
    chrono::Duration::seconds(ACTOR_REFETCH_INTERVAL_SECONDS_DEBUG)
  } else {
    chrono::Duration::seconds(ACTOR_REFETCH_INTERVAL_SECONDS)
  };
  last_refreshed.lt(&(naive_now() - update_interval))
}

/// Check if a remote community exists, create if not found, if its too old update it.Fetch a community, insert/update it in the database and return the community.
pub async fn get_or_fetch_and_upsert_community(
  apub_id: &Url,
  client: &Client,
  pool: &DbPool,
) -> Result<Community, LemmyError> {
  let apub_id_owned = apub_id.to_owned();
  let community = blocking(pool, move |conn| {
    Community::read_from_actor_id(conn, apub_id_owned.as_str())
  })
  .await?;

  match community {
    Ok(c) if !c.local && should_refetch_actor(c.last_refreshed_at) => {
      debug!("Fetching and updating from remote community: {}", apub_id);
      fetch_remote_community(apub_id, client, pool, Some(c.id)).await
    }
    Ok(c) => Ok(c),
    Err(NotFound {}) => {
      debug!("Fetching and creating remote community: {}", apub_id);
      fetch_remote_community(apub_id, client, pool, None).await
    }
    Err(e) => Err(e.into()),
  }
}

async fn fetch_remote_community(
  apub_id: &Url,
  client: &Client,
  pool: &DbPool,
  community_id: Option<i32>,
) -> Result<Community, LemmyError> {
  let group = fetch_remote_object::<GroupExt>(client, apub_id).await?;

  let cf = CommunityForm::from_apub(&group, client, pool).await?;
  let community = blocking(pool, move |conn| {
    if let Some(cid) = community_id {
      Community::update(conn, cid, &cf)
    } else {
      Community::create(conn, &cf)
    }
  })
  .await??;

  // Also add the community moderators too
  let attributed_to = group.inner.attributed_to().unwrap();
  let creator_and_moderator_uris: Vec<&Url> = attributed_to
    .as_many()
    .unwrap()
    .iter()
    .map(|a| a.as_xsd_any_uri().unwrap())
    .collect();

  let mut creator_and_moderators = Vec::new();

  for uri in creator_and_moderator_uris {
    let c_or_m = get_or_fetch_and_upsert_user(uri, client, pool).await?;

    creator_and_moderators.push(c_or_m);
  }

  // TODO: need to make this work to update mods of existing communities
  if community_id.is_none() {
    let community_id = community.id;
    blocking(pool, move |conn| {
      for mod_ in creator_and_moderators {
        let community_moderator_form = CommunityModeratorForm {
          community_id,
          user_id: mod_.id,
        };

        CommunityModerator::join(conn, &community_moderator_form)?;
      }
      Ok(()) as Result<(), LemmyError>
    })
    .await??;
  }

  // fetch outbox (maybe make this conditional)
  let outbox =
    fetch_remote_object::<OrderedCollection>(client, &community.get_outbox_url()?).await?;
  let outbox_items = outbox.items().clone();
  for o in outbox_items.many().unwrap() {
    let page = PageExt::from_any_base(o)?.unwrap();
    let post = PostForm::from_apub(&page, client, pool).await?;
    let post_ap_id = post.ap_id.clone();
    // Check whether the post already exists in the local db
    let existing = blocking(pool, move |conn| Post::read_from_apub_id(conn, &post_ap_id)).await?;
    match existing {
      Ok(e) => blocking(pool, move |conn| Post::update(conn, e.id, &post)).await??,
      Err(_) => blocking(pool, move |conn| Post::create(conn, &post)).await??,
    };
  }

  Ok(community)
}

fn upsert_post(post_form: &PostForm, conn: &PgConnection) -> Result<Post, LemmyError> {
  let existing = Post::read_from_apub_id(conn, &post_form.ap_id);
  match existing {
    Err(NotFound {}) => Ok(Post::create(conn, &post_form)?),
    Ok(p) => Ok(Post::update(conn, p.id, &post_form)?),
    Err(e) => Err(e.into()),
  }
}

pub async fn get_or_fetch_and_insert_post(
  post_ap_id: &Url,
  client: &Client,
  pool: &DbPool,
) -> Result<Post, LemmyError> {
  let post_ap_id_owned = post_ap_id.to_owned();
  let post = blocking(pool, move |conn| {
    Post::read_from_apub_id(conn, post_ap_id_owned.as_str())
  })
  .await?;

  match post {
    Ok(p) => Ok(p),
    Err(NotFound {}) => {
      debug!("Fetching and creating remote post: {}", post_ap_id);
      let post = fetch_remote_object::<PageExt>(client, post_ap_id).await?;
      let post_form = PostForm::from_apub(&post, client, pool).await?;

      let post = blocking(pool, move |conn| Post::create(conn, &post_form)).await??;

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

pub async fn get_or_fetch_and_insert_comment(
  comment_ap_id: &Url,
  client: &Client,
  pool: &DbPool,
) -> Result<Comment, LemmyError> {
  let comment_ap_id_owned = comment_ap_id.to_owned();
  let comment = blocking(pool, move |conn| {
    Comment::read_from_apub_id(conn, comment_ap_id_owned.as_str())
  })
  .await?;

  match comment {
    Ok(p) => Ok(p),
    Err(NotFound {}) => {
      debug!(
        "Fetching and creating remote comment and its parents: {}",
        comment_ap_id
      );
      let comment = fetch_remote_object::<Note>(client, comment_ap_id).await?;
      let comment_form = CommentForm::from_apub(&comment, client, pool).await?;

      let comment = blocking(pool, move |conn| Comment::create(conn, &comment_form)).await??;

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
