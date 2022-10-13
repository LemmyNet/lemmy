use crate::{
  fetcher::webfinger::webfinger_resolve_actor,
  local_instance,
  objects::{comment::ApubComment, community::ApubCommunity, person::ApubPerson, post::ApubPost},
  protocol::objects::{group::Group, note::Note, page::Page, person::Person},
};
use activitypub_federation::{core::object_id::ObjectId, traits::ApubObject};
use chrono::NaiveDateTime;
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;
use url::Url;

/// Converts search query to object id. The query can either be an URL, which will be treated as
/// ObjectId directly, or a webfinger identifier (@user@example.com or !community@example.com)
/// which gets resolved to an URL.
#[tracing::instrument(skip_all)]
pub async fn search_query_to_object_id(
  query: &str,
  local_only: bool,
  context: &LemmyContext,
) -> Result<SearchableObjects, LemmyError> {
  let request_counter = &mut 0;
  let object_id = match Url::parse(query) {
    // its already an url, just go with it
    Ok(url) => ObjectId::new(url),
    Err(_) => {
      // not an url, try to resolve via webfinger
      let mut chars = query.chars();
      let kind = chars.next();
      let identifier = chars.as_str();
      let id = match kind {
        Some('@') => {
          webfinger_resolve_actor::<ApubPerson>(identifier, local_only, context, request_counter)
            .await?
        }
        Some('!') => {
          webfinger_resolve_actor::<ApubCommunity>(identifier, local_only, context, request_counter)
            .await?
        }
        _ => return Err(LemmyError::from_message("invalid query")),
      };
      ObjectId::new(id)
    }
  };
  if local_only {
    object_id.dereference_local(context).await
  } else {
    object_id
      .dereference(context, local_instance(context), request_counter)
      .await
  }
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
  type DbType = ();
  type Error = LemmyError;

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
  #[tracing::instrument(skip_all)]
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

  #[tracing::instrument(skip_all)]
  async fn delete(self, data: &Self::DataType) -> Result<(), LemmyError> {
    match self {
      SearchableObjects::Person(p) => p.delete(data).await,
      SearchableObjects::Community(c) => c.delete(data).await,
      SearchableObjects::Post(p) => p.delete(data).await,
      SearchableObjects::Comment(c) => c.delete(data).await,
    }
  }

  async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, LemmyError> {
    unimplemented!()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    apub: &Self::ApubType,
    expected_domain: &Url,
    data: &Self::DataType,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    match apub {
      SearchableApubTypes::Group(a) => {
        ApubCommunity::verify(a, expected_domain, data, request_counter).await
      }
      SearchableApubTypes::Person(a) => {
        ApubPerson::verify(a, expected_domain, data, request_counter).await
      }
      SearchableApubTypes::Page(a) => {
        ApubPost::verify(a, expected_domain, data, request_counter).await
      }
      SearchableApubTypes::Note(a) => {
        ApubComment::verify(a, expected_domain, data, request_counter).await
      }
    }
  }

  #[tracing::instrument(skip_all)]
  async fn from_apub(
    apub: Self::ApubType,
    context: &LemmyContext,
    rc: &mut i32,
  ) -> Result<Self, LemmyError> {
    use SearchableApubTypes as SAT;
    use SearchableObjects as SO;
    Ok(match apub {
      SAT::Group(g) => SO::Community(ApubCommunity::from_apub(g, context, rc).await?),
      SAT::Person(p) => SO::Person(ApubPerson::from_apub(p, context, rc).await?),
      SAT::Page(p) => SO::Post(ApubPost::from_apub(p, context, rc).await?),
      SAT::Note(n) => SO::Comment(ApubComment::from_apub(n, context, rc).await?),
    })
  }
}
