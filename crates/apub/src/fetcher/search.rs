use crate::{
  fetcher::{
    fetch::fetch_remote_object,
    get_or_fetch_and_upsert_community,
    get_or_fetch_and_upsert_person,
    is_deleted,
  },
  find_object_by_id,
  objects::FromApub,
  GroupExt,
  NoteExt,
  Object,
  PageExt,
  PersonExt,
};
use activitystreams::base::BaseExt;
use anyhow::{anyhow, Context};
use lemmy_api_common::{blocking, site::SearchResponse};
use lemmy_db_queries::{
  source::{
    comment::Comment_,
    community::Community_,
    person::Person_,
    post::Post_,
    private_message::PrivateMessage_,
  },
  SearchType,
};
use lemmy_db_schema::source::{
  comment::Comment,
  community::Community,
  person::Person,
  post::Post,
  private_message::PrivateMessage,
};
use lemmy_db_views::{comment_view::CommentView, post_view::PostView};
use lemmy_db_views_actor::{community_view::CommunityView, person_view::PersonViewSafe};
use lemmy_utils::{settings::structs::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use log::debug;
use url::Url;

/// The types of ActivityPub objects that can be fetched directly by searching for their ID.
#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
enum SearchAcceptedObjects {
  Person(Box<PersonExt>),
  Group(Box<GroupExt>),
  Page(Box<PageExt>),
  Comment(Box<NoteExt>),
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

    // Person type will look like ['', username, instance]
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

  let recursion_counter = &mut 0;
  let fetch_response =
    fetch_remote_object::<SearchAcceptedObjects>(context.client(), &query_url, recursion_counter)
      .await;
  if is_deleted(&fetch_response) {
    delete_object_locally(&query_url, context).await?;
  }

  // Necessary because we get a stack overflow using FetchError
  let fet_res = fetch_response.map_err(|e| LemmyError::from(e.inner))?;
  build_response(fet_res, query_url, recursion_counter, context).await
}

async fn build_response(
  fetch_response: SearchAcceptedObjects,
  query_url: Url,
  recursion_counter: &mut i32,
  context: &LemmyContext,
) -> Result<SearchResponse, LemmyError> {
  let domain = query_url.domain().context("url has no domain")?;
  let mut response = SearchResponse {
    type_: SearchType::All.to_string(),
    comments: vec![],
    posts: vec![],
    communities: vec![],
    users: vec![],
  };

  match fetch_response {
    SearchAcceptedObjects::Person(p) => {
      let person_uri = p.inner.id(domain)?.context("person has no id")?;

      let person = get_or_fetch_and_upsert_person(&person_uri, context, recursion_counter).await?;

      response.users = vec![
        blocking(context.pool(), move |conn| {
          PersonViewSafe::read(conn, person.id)
        })
        .await??,
      ];
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
    }
    SearchAcceptedObjects::Page(p) => {
      let p = Post::from_apub(&p, context, query_url, recursion_counter, false).await?;

      response.posts =
        vec![blocking(context.pool(), move |conn| PostView::read(conn, p.id, None)).await??];
    }
    SearchAcceptedObjects::Comment(c) => {
      let c = Comment::from_apub(&c, context, query_url, recursion_counter, false).await?;

      response.comments = vec![
        blocking(context.pool(), move |conn| {
          CommentView::read(conn, c.id, None)
        })
        .await??,
      ];
    }
  };

  Ok(response)
}

async fn delete_object_locally(query_url: &Url, context: &LemmyContext) -> Result<(), LemmyError> {
  let res = find_object_by_id(context, query_url.to_owned()).await?;
  match res {
    Object::Comment(c) => {
      blocking(context.pool(), move |conn| {
        Comment::update_deleted(conn, c.id, true)
      })
      .await??;
    }
    Object::Post(p) => {
      blocking(context.pool(), move |conn| {
        Post::update_deleted(conn, p.id, true)
      })
      .await??;
    }
    Object::Person(u) => {
      // TODO: implement update_deleted() for user, move it to ApubObject trait
      blocking(context.pool(), move |conn| {
        Person::delete_account(conn, u.id)
      })
      .await??;
    }
    Object::Community(c) => {
      blocking(context.pool(), move |conn| {
        Community::update_deleted(conn, c.id, true)
      })
      .await??;
    }
    Object::PrivateMessage(pm) => {
      blocking(context.pool(), move |conn| {
        PrivateMessage::update_deleted(conn, pm.id, true)
      })
      .await??;
    }
  }
  Err(anyhow!("Object was deleted").into())
}
