use crate::{
  check_is_apub_id_valid,
  ActorType,
  FromApub,
  GroupExt,
  PageExt,
  PersonExt,
  APUB_JSON_CONTENT_TYPE,
};
use activitystreams::{base::BaseExt, collection::OrderedCollection, object::Note, prelude::*};
use anyhow::{anyhow, Context};
use chrono::NaiveDateTime;
use diesel::result::Error::NotFound;
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
use lemmy_structs::{blocking, site::SearchResponse};
use lemmy_utils::{
  location_info,
  request::{retry, RecvError},
  settings::Settings,
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use log::debug;
use reqwest::Client;
use serde::Deserialize;
use std::{fmt::Debug, time::Duration};
use url::Url;

static ACTOR_REFETCH_INTERVAL_SECONDS: i64 = 24 * 60 * 60;
static ACTOR_REFETCH_INTERVAL_SECONDS_DEBUG: i64 = 10;

/// Maximum number of HTTP requests allowed to handle a single incoming activity (or a single object
/// fetch through the search).
///
/// Tests are passing with a value of 5, so 10 should be safe for production.
static MAX_REQUEST_NUMBER: i32 = 10;

/// Fetch any type of ActivityPub object, handling things like HTTP headers, deserialisation,
/// timeouts etc.
async fn fetch_remote_object<Response>(
  client: &Client,
  url: &Url,
  recursion_counter: &mut i32,
) -> Result<Response, LemmyError>
where
  Response: for<'de> Deserialize<'de>,
{
  *recursion_counter += 1;
  if *recursion_counter > MAX_REQUEST_NUMBER {
    return Err(anyhow!("Maximum recursion depth reached").into());
  }
  check_is_apub_id_valid(&url)?;

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
/// Some working examples for use with the `docker/federation/` setup:
/// http://lemmy_alpha:8541/c/main, or !main@lemmy_alpha:8541
/// http://lemmy_beta:8551/u/lemmy_alpha, or @lemmy_beta@lemmy_beta:8551
/// http://lemmy_gamma:8561/post/3
/// http://lemmy_delta:8571/comment/2
pub async fn search_by_apub_id(
  query: &str,
  context: &LemmyContext,
) -> Result<SearchResponse, LemmyError> {
  // Parse the shorthand query url
  let query_url = if query.contains('@') {
    debug!("Search for {}", query);
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
        return Err(anyhow!("Invalid search query: {}", query).into());
      }
    } else {
      return Err(anyhow!("Invalid search query: {}", query).into());
    };

    let url = format!(
      "{}://{}{}",
      Settings::get().get_protocol_string(),
      instance,
      name
    );
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

  let domain = query_url.domain().context("url has no domain")?;
  let recursion_counter = &mut 0;
  let response = match fetch_remote_object::<SearchAcceptedObjects>(
    context.client(),
    &query_url,
    recursion_counter,
  )
  .await?
  {
    SearchAcceptedObjects::Person(p) => {
      let user_uri = p.inner.id(domain)?.context("person has no id")?;

      let user = get_or_fetch_and_upsert_user(&user_uri, context, recursion_counter).await?;

      response.users = vec![
        blocking(context.pool(), move |conn| {
          UserView::get_user_secure(conn, user.id)
        })
        .await??,
      ];

      response
    }
    SearchAcceptedObjects::Group(g) => {
      let community_uri = g.inner.id(domain)?.context("group has no id")?;

      let community =
        get_or_fetch_and_upsert_community(community_uri, context, recursion_counter).await?;

      response.communities = vec![
        blocking(context.pool(), move |conn| {
          CommunityView::read(conn, community.id, None)
        })
        .await??,
      ];

      response
    }
    SearchAcceptedObjects::Page(p) => {
      let post_form = PostForm::from_apub(&p, context, Some(query_url), recursion_counter).await?;

      let p = blocking(context.pool(), move |conn| Post::upsert(conn, &post_form)).await??;
      response.posts =
        vec![blocking(context.pool(), move |conn| PostView::read(conn, p.id, None)).await??];

      response
    }
    SearchAcceptedObjects::Comment(c) => {
      let comment_form =
        CommentForm::from_apub(&c, context, Some(query_url), recursion_counter).await?;

      let c = blocking(context.pool(), move |conn| {
        Comment::upsert(conn, &comment_form)
      })
      .await??;
      response.comments = vec![
        blocking(context.pool(), move |conn| {
          CommentView::read(conn, c.id, None)
        })
        .await??,
      ];

      response
    }
  };

  Ok(response)
}

/// Get a remote actor from its apub ID (either a user or a community). Thin wrapper around
/// `get_or_fetch_and_upsert_user()` and `get_or_fetch_and_upsert_community()`.
///
/// If it exists locally and `!should_refetch_actor()`, it is returned directly from the database.
/// Otherwise it is fetched from the remote instance, stored and returned.
pub(crate) async fn get_or_fetch_and_upsert_actor(
  apub_id: &Url,
  context: &LemmyContext,
  recursion_counter: &mut i32,
) -> Result<Box<dyn ActorType>, LemmyError> {
  let community = get_or_fetch_and_upsert_community(apub_id, context, recursion_counter).await;
  let actor: Box<dyn ActorType> = match community {
    Ok(c) => Box::new(c),
    Err(_) => Box::new(get_or_fetch_and_upsert_user(apub_id, context, recursion_counter).await?),
  };
  Ok(actor)
}

/// Get a user from its apub ID.
///
/// If it exists locally and `!should_refetch_actor()`, it is returned directly from the database.
/// Otherwise it is fetched from the remote instance, stored and returned.
pub(crate) async fn get_or_fetch_and_upsert_user(
  apub_id: &Url,
  context: &LemmyContext,
  recursion_counter: &mut i32,
) -> Result<User_, LemmyError> {
  let apub_id_owned = apub_id.to_owned();
  let user = blocking(context.pool(), move |conn| {
    User_::read_from_actor_id(conn, apub_id_owned.as_ref())
  })
  .await?;

  match user {
    // If its older than a day, re-fetch it
    Ok(u) if !u.local && should_refetch_actor(u.last_refreshed_at) => {
      debug!("Fetching and updating from remote user: {}", apub_id);
      let person =
        fetch_remote_object::<PersonExt>(context.client(), apub_id, recursion_counter).await;
      // If fetching failed, return the existing data.
      if person.is_err() {
        return Ok(u);
      }

      let mut uf = UserForm::from_apub(
        &person?,
        context,
        Some(apub_id.to_owned()),
        recursion_counter,
      )
      .await?;
      uf.last_refreshed_at = Some(naive_now());
      let user = blocking(context.pool(), move |conn| User_::update(conn, u.id, &uf)).await??;

      Ok(user)
    }
    Ok(u) => Ok(u),
    Err(NotFound {}) => {
      debug!("Fetching and creating remote user: {}", apub_id);
      let person =
        fetch_remote_object::<PersonExt>(context.client(), apub_id, recursion_counter).await?;

      let uf = UserForm::from_apub(
        &person,
        context,
        Some(apub_id.to_owned()),
        recursion_counter,
      )
      .await?;
      let user = blocking(context.pool(), move |conn| User_::upsert(conn, &uf)).await??;

      Ok(user)
    }
    Err(e) => Err(e.into()),
  }
}

/// Determines when a remote actor should be refetched from its instance. In release builds, this is
/// `ACTOR_REFETCH_INTERVAL_SECONDS` after the last refetch, in debug builds
/// `ACTOR_REFETCH_INTERVAL_SECONDS_DEBUG`.
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

/// Get a community from its apub ID.
///
/// If it exists locally and `!should_refetch_actor()`, it is returned directly from the database.
/// Otherwise it is fetched from the remote instance, stored and returned.
pub(crate) async fn get_or_fetch_and_upsert_community(
  apub_id: &Url,
  context: &LemmyContext,
  recursion_counter: &mut i32,
) -> Result<Community, LemmyError> {
  let apub_id_owned = apub_id.to_owned();
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_actor_id(conn, apub_id_owned.as_str())
  })
  .await?;

  match community {
    Ok(c) if !c.local && should_refetch_actor(c.last_refreshed_at) => {
      debug!("Fetching and updating from remote community: {}", apub_id);
      fetch_remote_community(apub_id, context, Some(c), recursion_counter).await
    }
    Ok(c) => Ok(c),
    Err(NotFound {}) => {
      debug!("Fetching and creating remote community: {}", apub_id);
      fetch_remote_community(apub_id, context, None, recursion_counter).await
    }
    Err(e) => Err(e.into()),
  }
}

/// Request a community by apub ID from a remote instance, including moderators. If `old_community`,
/// is set, this is an update for a community which is already known locally. If not, we don't know
/// the community yet and also pull the outbox, to get some initial posts.
async fn fetch_remote_community(
  apub_id: &Url,
  context: &LemmyContext,
  old_community: Option<Community>,
  recursion_counter: &mut i32,
) -> Result<Community, LemmyError> {
  let group = fetch_remote_object::<GroupExt>(context.client(), apub_id, recursion_counter).await;
  // If fetching failed, return the existing data.
  if let Some(ref c) = old_community {
    if group.is_err() {
      return Ok(c.to_owned());
    }
  }

  let group = group?;
  let cf =
    CommunityForm::from_apub(&group, context, Some(apub_id.to_owned()), recursion_counter).await?;
  let community = blocking(context.pool(), move |conn| Community::upsert(conn, &cf)).await??;

  // Also add the community moderators too
  let attributed_to = group.inner.attributed_to().context(location_info!())?;
  let creator_and_moderator_uris: Vec<&Url> = attributed_to
    .as_many()
    .context(location_info!())?
    .iter()
    .map(|a| a.as_xsd_any_uri().context(""))
    .collect::<Result<Vec<&Url>, anyhow::Error>>()?;

  let mut creator_and_moderators = Vec::new();

  for uri in creator_and_moderator_uris {
    let c_or_m = get_or_fetch_and_upsert_user(uri, context, recursion_counter).await?;

    creator_and_moderators.push(c_or_m);
  }

  // TODO: need to make this work to update mods of existing communities
  if old_community.is_none() {
    let community_id = community.id;
    blocking(context.pool(), move |conn| {
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
  let outbox = fetch_remote_object::<OrderedCollection>(
    context.client(),
    &community.get_outbox_url()?,
    recursion_counter,
  )
  .await?;
  let outbox_items = outbox.items().context(location_info!())?.clone();
  let mut outbox_items = outbox_items.many().context(location_info!())?;
  if outbox_items.len() > 20 {
    outbox_items = outbox_items[0..20].to_vec();
  }
  for o in outbox_items {
    let page = PageExt::from_any_base(o)?.context(location_info!())?;

    // The post creator may be from a blocked instance,
    // if it errors, then continue
    let post = match PostForm::from_apub(&page, context, None, recursion_counter).await {
      Ok(post) => post,
      Err(_) => continue,
    };
    let post_ap_id = post.ap_id.as_ref().context(location_info!())?.clone();
    // Check whether the post already exists in the local db
    let existing = blocking(context.pool(), move |conn| {
      Post::read_from_apub_id(conn, &post_ap_id)
    })
    .await?;
    match existing {
      Ok(e) => blocking(context.pool(), move |conn| Post::update(conn, e.id, &post)).await??,
      Err(_) => blocking(context.pool(), move |conn| Post::upsert(conn, &post)).await??,
    };
    // TODO: we need to send a websocket update here
  }

  Ok(community)
}

/// Gets a post by its apub ID. If it exists locally, it is returned directly. Otherwise it is
/// pulled from its apub ID, inserted and returned.
///
/// The parent community is also pulled if necessary. Comments are not pulled.
pub(crate) async fn get_or_fetch_and_insert_post(
  post_ap_id: &Url,
  context: &LemmyContext,
  recursion_counter: &mut i32,
) -> Result<Post, LemmyError> {
  let post_ap_id_owned = post_ap_id.to_owned();
  let post = blocking(context.pool(), move |conn| {
    Post::read_from_apub_id(conn, post_ap_id_owned.as_str())
  })
  .await?;

  match post {
    Ok(p) => Ok(p),
    Err(NotFound {}) => {
      debug!("Fetching and creating remote post: {}", post_ap_id);
      let post =
        fetch_remote_object::<PageExt>(context.client(), post_ap_id, recursion_counter).await?;
      let post_form = PostForm::from_apub(
        &post,
        context,
        Some(post_ap_id.to_owned()),
        recursion_counter,
      )
      .await?;

      let post = blocking(context.pool(), move |conn| Post::upsert(conn, &post_form)).await??;

      Ok(post)
    }
    Err(e) => Err(e.into()),
  }
}

/// Gets a comment by its apub ID. If it exists locally, it is returned directly. Otherwise it is
/// pulled from its apub ID, inserted and returned.
///
/// The parent community, post and comment are also pulled if necessary.
pub(crate) async fn get_or_fetch_and_insert_comment(
  comment_ap_id: &Url,
  context: &LemmyContext,
  recursion_counter: &mut i32,
) -> Result<Comment, LemmyError> {
  let comment_ap_id_owned = comment_ap_id.to_owned();
  let comment = blocking(context.pool(), move |conn| {
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
      let comment =
        fetch_remote_object::<Note>(context.client(), comment_ap_id, recursion_counter).await?;
      let comment_form = CommentForm::from_apub(
        &comment,
        context,
        Some(comment_ap_id.to_owned()),
        recursion_counter,
      )
      .await?;

      let comment = blocking(context.pool(), move |conn| {
        Comment::upsert(conn, &comment_form)
      })
      .await??;

      Ok(comment)
    }
    Err(e) => Err(e.into()),
  }
}
