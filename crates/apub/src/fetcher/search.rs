use crate::{
  fetcher::{deletable_apub_object::DeletableApubObject, object_id::ObjectId},
  objects::{comment::Note, community::Group, person::Person as ApubPerson, post::Page, FromApub},
};
use activitystreams::chrono::NaiveDateTime;
use anyhow::anyhow;
use diesel::{result::Error, PgConnection};
use itertools::Itertools;
use lemmy_api_common::blocking;
use lemmy_apub_lib::webfinger::{webfinger_resolve_actor, WebfingerType};
use lemmy_db_queries::{
  source::{community::Community_, person::Person_},
  ApubObject,
  DbPool,
};
use lemmy_db_schema::{
  source::{comment::Comment, community::Community, person::Person, post::Post},
  DbUrl,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;
use url::Url;

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
) -> Result<SearchableObjects, LemmyError> {
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
        webfinger_resolve_actor(
          name,
          domain,
          kind,
          context.client(),
          context.settings().get_protocol_string(),
        )
        .await?
      }
      // local actor, read from database and return
      else {
        return find_local_actor_by_name(name, kind, context.pool()).await;
      }
    }
  };

  let request_counter = &mut 0;
  ObjectId::new(query_url)
    .dereference(context, request_counter)
    .await
}

async fn find_local_actor_by_name(
  name: &str,
  kind: WebfingerType,
  pool: &DbPool,
) -> Result<SearchableObjects, LemmyError> {
  let name: String = name.into();
  Ok(match kind {
    WebfingerType::Group => SearchableObjects::Community(
      blocking(pool, move |conn| Community::read_from_name(conn, &name)).await??,
    ),
    WebfingerType::Person => SearchableObjects::Person(
      blocking(pool, move |conn| Person::find_by_name(conn, &name)).await??,
    ),
  })
}

/// The types of ActivityPub objects that can be fetched directly by searching for their ID.
#[derive(Debug)]
pub enum SearchableObjects {
  Person(Person),
  Community(Community),
  Post(Post),
  Comment(Comment),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum SearchableApubTypes {
  Group(Group),
  Person(ApubPerson),
  Page(Page),
  Note(Note),
}

impl ApubObject for SearchableObjects {
  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    match self {
      SearchableObjects::Person(p) => p.last_refreshed_at(),
      SearchableObjects::Community(c) => c.last_refreshed_at(),
      SearchableObjects::Post(p) => p.last_refreshed_at(),
      SearchableObjects::Comment(c) => c.last_refreshed_at(),
    }
  }

  // TODO: this is inefficient, because if the object is not in local db, it will run 4 db queries
  //       before finally returning an error. it would be nice if we could check all 4 tables in
  //       a single query.
  //       we could skip this and always return an error, but then it would not be able to mark
  //       objects as deleted that were deleted by remote server.
  fn read_from_apub_id(conn: &PgConnection, object_id: &DbUrl) -> Result<Self, Error> {
    let c = Community::read_from_apub_id(conn, object_id);
    if let Ok(c) = c {
      return Ok(SearchableObjects::Community(c));
    }
    let p = Person::read_from_apub_id(conn, object_id);
    if let Ok(p) = p {
      return Ok(SearchableObjects::Person(p));
    }
    let p = Post::read_from_apub_id(conn, object_id);
    if let Ok(p) = p {
      return Ok(SearchableObjects::Post(p));
    }
    let c = Comment::read_from_apub_id(conn, object_id);
    Ok(SearchableObjects::Comment(c?))
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for SearchableObjects {
  type ApubType = SearchableApubTypes;

  async fn from_apub(
    apub: &Self::ApubType,
    context: &LemmyContext,
    ed: &Url,
    rc: &mut i32,
  ) -> Result<Self, LemmyError> {
    use SearchableApubTypes as SAT;
    use SearchableObjects as SO;
    Ok(match apub {
      SAT::Group(g) => SO::Community(Community::from_apub(g, context, ed, rc).await?),
      SAT::Person(p) => SO::Person(Person::from_apub(p, context, ed, rc).await?),
      SAT::Page(p) => SO::Post(Post::from_apub(p, context, ed, rc).await?),
      SAT::Note(n) => SO::Comment(Comment::from_apub(n, context, ed, rc).await?),
    })
  }
}

#[async_trait::async_trait(?Send)]
impl DeletableApubObject for SearchableObjects {
  async fn delete(self, context: &LemmyContext) -> Result<(), LemmyError> {
    match self {
      SearchableObjects::Person(p) => p.delete(context).await,
      SearchableObjects::Community(c) => c.delete(context).await,
      SearchableObjects::Post(p) => p.delete(context).await,
      SearchableObjects::Comment(c) => c.delete(context).await,
    }
  }
}
