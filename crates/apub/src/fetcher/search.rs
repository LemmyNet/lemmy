use crate::{
  fetcher::{
    community::get_or_fetch_and_upsert_community,
    fetch::fetch_remote_object,
    is_deleted,
    person::get_or_fetch_and_upsert_person,
  },
  find_object_by_id,
  objects::{comment::Note, community::Group, person::Person as ApubPerson, post::Page, FromApub},
  Object,
};
use anyhow::anyhow;
use itertools::Itertools;
use lemmy_api_common::{blocking, site::ResolveObjectResponse};
use lemmy_apub_lib::webfinger::{webfinger_resolve_actor, WebfingerType};
use lemmy_db_queries::source::{
  comment::Comment_,
  community::Community_,
  person::Person_,
  post::Post_,
  private_message::PrivateMessage_,
};
use lemmy_db_schema::source::{
  comment::Comment,
  community::Community,
  person::Person,
  post::Post,
  private_message::PrivateMessage,
};
use lemmy_db_views::{
  comment_view::CommentView,
  local_user_view::LocalUserView,
  post_view::PostView,
};
use lemmy_db_views_actor::{community_view::CommunityView, person_view::PersonViewSafe};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

/// The types of ActivityPub objects that can be fetched directly by searching for their ID.
#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
enum SearchAcceptedObjects {
  Person(Box<ApubPerson>),
  Group(Box<Group>),
  Page(Box<Page>),
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
  local_user_view: Option<LocalUserView>,
  context: &LemmyContext,
) -> Result<ResolveObjectResponse, LemmyError> {
  let query_url = match Url::parse(query) {
    Ok(u) => u,
    Err(_) => {
      let (kind, name) = query.split_at(1);
      let kind = match kind {
        "@" => WebfingerType::Person,
        "!" => WebfingerType::Group,
        _ => return Err(anyhow!("invalid query").into()),
      };
      // remote actor, use webfinger to resolve url
      if name.contains('@') {
        let (name, domain) = name.splitn(2, '@').collect_tuple().expect("invalid query");
        webfinger_resolve_actor(name, domain, kind, context.client()).await?
      }
      // local actor, read from database and return
      else {
        let name: String = name.into();
        return match kind {
          WebfingerType::Group => {
            let res = blocking(context.pool(), move |conn| {
              let community = Community::read_from_name(conn, &name)?;
              CommunityView::read(conn, community.id, local_user_view.map(|l| l.person.id))
            })
            .await??;
            Ok(ResolveObjectResponse {
              community: Some(res),
              ..ResolveObjectResponse::default()
            })
          }
          WebfingerType::Person => {
            let res = blocking(context.pool(), move |conn| {
              let person = Person::find_by_name(conn, &name)?;
              PersonViewSafe::read(conn, person.id)
            })
            .await??;
            Ok(ResolveObjectResponse {
              person: Some(res),
              ..ResolveObjectResponse::default()
            })
          }
        };
      }
    }
  };

  let request_counter = &mut 0;
  // this does a fetch (even for local objects), just to determine its type and fetch it again
  // below. we need to fix this when rewriting the fetcher.
  let fetch_response =
    fetch_remote_object::<SearchAcceptedObjects>(context.client(), &query_url, request_counter)
      .await;
  if is_deleted(&fetch_response) {
    delete_object_locally(&query_url, context).await?;
    return Err(anyhow!("Object was deleted").into());
  }

  // Necessary because we get a stack overflow using FetchError
  let fet_res = fetch_response.map_err(|e| LemmyError::from(e.inner))?;
  build_response(fet_res, query_url, request_counter, context).await
}

async fn build_response(
  fetch_response: SearchAcceptedObjects,
  query_url: Url,
  recursion_counter: &mut i32,
  context: &LemmyContext,
) -> Result<ResolveObjectResponse, LemmyError> {
  use ResolveObjectResponse as ROR;
  Ok(match fetch_response {
    SearchAcceptedObjects::Person(p) => {
      let person_uri = p.id(&query_url)?;

      let person = get_or_fetch_and_upsert_person(person_uri, context, recursion_counter).await?;
      ROR {
        person: blocking(context.pool(), move |conn| {
          PersonViewSafe::read(conn, person.id)
        })
        .await?
        .ok(),
        ..ROR::default()
      }
    }
    SearchAcceptedObjects::Group(g) => {
      let community_uri = g.id(&query_url)?;
      let community =
        get_or_fetch_and_upsert_community(community_uri, context, recursion_counter).await?;
      ROR {
        community: blocking(context.pool(), move |conn| {
          CommunityView::read(conn, community.id, None)
        })
        .await?
        .ok(),
        ..ROR::default()
      }
    }
    SearchAcceptedObjects::Page(p) => {
      let p = Post::from_apub(&p, context, &query_url, recursion_counter).await?;
      ROR {
        post: blocking(context.pool(), move |conn| PostView::read(conn, p.id, None))
          .await?
          .ok(),
        ..ROR::default()
      }
    }
    SearchAcceptedObjects::Comment(c) => {
      let c = Comment::from_apub(&c, context, &query_url, recursion_counter).await?;
      ROR {
        comment: blocking(context.pool(), move |conn| {
          CommentView::read(conn, c.id, None)
        })
        .await?
        .ok(),
        ..ROR::default()
      }
    }
  })
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
  Ok(())
}
