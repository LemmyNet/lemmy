use crate::{
  fetcher::webfinger::webfinger_resolve,
  objects::{comment::ApubComment, community::ApubCommunity, person::ApubPerson, post::ApubPost},
  protocol::objects::{group::Group, note::Note, page::Page, person::Person},
  EndpointType,
};
use anyhow::anyhow;
use chrono::NaiveDateTime;
use lemmy_apub_lib::{object_id::ObjectId, traits::ApubObject};
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
  let request_counter = &mut 0;
  match Url::parse(query) {
    Ok(url) => {
      ObjectId::new(url)
        .dereference(context, request_counter)
        .await
    }
    Err(_) => {
      let (kind, identifier) = query.split_at(1);
      match kind {
        "@" => {
          let id = webfinger_resolve::<ApubPerson>(
            identifier,
            EndpointType::Person,
            context,
            request_counter,
          )
          .await?;
          Ok(SearchableObjects::Person(
            ObjectId::new(id)
              .dereference(context, request_counter)
              .await?,
          ))
        }
        "!" => {
          let id = webfinger_resolve::<ApubCommunity>(
            identifier,
            EndpointType::Community,
            context,
            request_counter,
          )
          .await?;
          Ok(SearchableObjects::Community(
            ObjectId::new(id)
              .dereference(context, request_counter)
              .await?,
          ))
        }
        _ => Err(anyhow!("invalid query").into()),
      }
    }
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
  Note(Box<Note>),
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

  async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, LemmyError> {
    unimplemented!()
  }

  fn to_tombstone(&self) -> Result<Self::TombstoneType, LemmyError> {
    unimplemented!()
  }

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
      SAT::Note(n) => SO::Comment(ApubComment::from_apub(*n, context, rc).await?),
    })
  }
}
