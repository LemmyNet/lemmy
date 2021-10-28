use anyhow::anyhow;
use chrono::NaiveDateTime;
use itertools::Itertools;
use serde::Deserialize;
use url::Url;

use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  traits::ApubObject,
  webfinger::{webfinger_resolve_actor, WebfingerType},
};
use lemmy_db_schema::{
  source::{community::Community, person::Person as DbPerson},
  DbPool,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;

use crate::{
  fetcher::object_id::ObjectId,
  objects::{
    comment::ApubComment,
    community::ApubCommunity,
    person::{ApubPerson, Person},
    post::ApubPost,
  },
  protocol::objects::{group::Group, note::Note, page::Page},
};

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
      blocking(pool, move |conn| Community::read_from_name(conn, &name))
        .await??
        .into(),
    ),
    WebfingerType::Person => SearchableObjects::Person(
      blocking(pool, move |conn| DbPerson::find_by_name(conn, &name))
        .await??
        .into(),
    ),
  })
}

/// The types of ActivityPub objects that can be fetched directly by searching for their ID.
#[derive(Debug)]
pub enum SearchableObjects {
  Person(ApubPerson),
  Community(ApubCommunity),
  Post(ApubPost),
  Comment(ApubComment),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum SearchableApubTypes {
  Group(Group),
  Person(Person),
  Page(Page),
  Note(Note),
}

#[async_trait::async_trait(?Send)]
impl ApubObject for SearchableObjects {
  type DataType = LemmyContext;
  type ApubType = SearchableApubTypes;
  type TombstoneType = ();

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
  //       we could skip this and always return an error, but then it would always fetch objects
  //       over http, and not be able to mark objects as deleted that were deleted by remote server.
  async fn read_from_apub_id(
    object_id: Url,
    context: &LemmyContext,
  ) -> Result<Option<Self>, LemmyError> {
    let c = ApubCommunity::read_from_apub_id(object_id.clone(), context).await?;
    if let Some(c) = c {
      return Ok(Some(SearchableObjects::Community(c)));
    }
    let p = ApubPerson::read_from_apub_id(object_id.clone(), context).await?;
    if let Some(p) = p {
      return Ok(Some(SearchableObjects::Person(p)));
    }
    let p = ApubPost::read_from_apub_id(object_id.clone(), context).await?;
    if let Some(p) = p {
      return Ok(Some(SearchableObjects::Post(p)));
    }
    let c = ApubComment::read_from_apub_id(object_id, context).await?;
    if let Some(c) = c {
      return Ok(Some(SearchableObjects::Comment(c)));
    }
    Ok(None)
  }

  async fn delete(self, data: &Self::DataType) -> Result<(), LemmyError> {
    match self {
      SearchableObjects::Person(p) => p.delete(data).await,
      SearchableObjects::Community(c) => c.delete(data).await,
      SearchableObjects::Post(p) => p.delete(data).await,
      SearchableObjects::Comment(c) => c.delete(data).await,
    }
  }

  async fn to_apub(&self, _data: &Self::DataType) -> Result<Self::ApubType, LemmyError> {
    unimplemented!()
  }

  fn to_tombstone(&self) -> Result<Self::TombstoneType, LemmyError> {
    unimplemented!()
  }

  async fn from_apub(
    apub: &Self::ApubType,
    context: &LemmyContext,
    ed: &Url,
    rc: &mut i32,
  ) -> Result<Self, LemmyError> {
    use SearchableApubTypes as SAT;
    use SearchableObjects as SO;
    Ok(match apub {
      SAT::Group(g) => SO::Community(ApubCommunity::from_apub(g, context, ed, rc).await?),
      SAT::Person(p) => SO::Person(ApubPerson::from_apub(p, context, ed, rc).await?),
      SAT::Page(p) => SO::Post(ApubPost::from_apub(p, context, ed, rc).await?),
      SAT::Note(n) => SO::Comment(ApubComment::from_apub(n, context, ed, rc).await?),
    })
  }
}
