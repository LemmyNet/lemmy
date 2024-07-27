use crate::{
  fetcher::user_or_community::{PersonOrGroup, UserOrCommunity},
  objects::{comment::ApubComment, community::ApubCommunity, person::ApubPerson, post::ApubPost},
  protocol::objects::{note::Note, page::Page},
};
use activitypub_federation::{
  config::Data,
  fetch::{object_id::ObjectId, webfinger::webfinger_resolve_actor},
  traits::Object,
};
use chrono::{DateTime, Utc};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::{LemmyError, LemmyResult};
use serde::Deserialize;
use url::Url;

/// Converts search query to object id. The query can either be an URL, which will be treated as
/// ObjectId directly, or a webfinger identifier (@user@example.com or !community@example.com)
/// which gets resolved to an URL.
#[tracing::instrument(skip_all)]
pub(crate) async fn search_query_to_object_id(
  mut query: String,
  context: &Data<LemmyContext>,
) -> LemmyResult<SearchableObjects> {
  Ok(match Url::parse(&query) {
    Ok(url) => {
      // its already an url, just go with it
      ObjectId::from(url).dereference(context).await?
    }
    Err(_) => {
      // not an url, try to resolve via webfinger
      if query.starts_with('!') || query.starts_with('@') {
        query.remove(0);
      }
      SearchableObjects::PersonOrCommunity(Box::new(
        webfinger_resolve_actor::<LemmyContext, UserOrCommunity>(&query, context).await?,
      ))
    }
  })
}

/// Converts a search query to an object id.  The query MUST bbe a URL which will bbe treated
/// as the ObjectId directly.  If the query is a webfinger identifier (@user@example.com or
/// !community@example.com) this method will return an error.
#[tracing::instrument(skip_all)]
pub(crate) async fn search_query_to_object_id_local(
  query: &str,
  context: &Data<LemmyContext>,
) -> LemmyResult<SearchableObjects> {
  let url = Url::parse(query)?;
  ObjectId::from(url).dereference_local(context).await
}

/// The types of ActivityPub objects that can be fetched directly by searching for their ID.
#[derive(Debug)]
pub(crate) enum SearchableObjects {
  Post(ApubPost),
  Comment(ApubComment),
  PersonOrCommunity(Box<UserOrCommunity>),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum SearchableKinds {
  Page(Box<Page>),
  Note(Note),
  PersonOrGroup(Box<PersonOrGroup>),
}

#[async_trait::async_trait]
impl Object for SearchableObjects {
  type DataType = LemmyContext;
  type Kind = SearchableKinds;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    match self {
      SearchableObjects::Post(p) => p.last_refreshed_at(),
      SearchableObjects::Comment(c) => c.last_refreshed_at(),
      SearchableObjects::PersonOrCommunity(p) => p.last_refreshed_at(),
    }
  }

  // TODO: this is inefficient, because if the object is not in local db, it will run 4 db queries
  //       before finally returning an error. it would be nice if we could check all 4 tables in
  //       a single query.
  //       we could skip this and always return an error, but then it would always fetch objects
  //       over http, and not be able to mark objects as deleted that were deleted by remote server.
  #[tracing::instrument(skip_all)]
  async fn read_from_id(
    object_id: Url,
    context: &Data<Self::DataType>,
  ) -> LemmyResult<Option<Self>> {
    let uc = UserOrCommunity::read_from_id(object_id.clone(), context).await?;
    if let Some(uc) = uc {
      return Ok(Some(SearchableObjects::PersonOrCommunity(Box::new(uc))));
    }
    let p = ApubPost::read_from_id(object_id.clone(), context).await?;
    if let Some(p) = p {
      return Ok(Some(SearchableObjects::Post(p)));
    }
    let c = ApubComment::read_from_id(object_id, context).await?;
    if let Some(c) = c {
      return Ok(Some(SearchableObjects::Comment(c)));
    }
    Ok(None)
  }

  #[tracing::instrument(skip_all)]
  async fn delete(self, data: &Data<Self::DataType>) -> LemmyResult<()> {
    match self {
      SearchableObjects::Post(p) => p.delete(data).await,
      SearchableObjects::Comment(c) => c.delete(data).await,
      SearchableObjects::PersonOrCommunity(pc) => match *pc {
        UserOrCommunity::User(p) => p.delete(data).await,
        UserOrCommunity::Community(c) => c.delete(data).await,
      },
    }
  }

  async fn into_json(self, data: &Data<Self::DataType>) -> LemmyResult<Self::Kind> {
    Ok(match self {
      SearchableObjects::Post(p) => SearchableKinds::Page(Box::new(p.into_json(data).await?)),
      SearchableObjects::Comment(c) => SearchableKinds::Note(c.into_json(data).await?),
      SearchableObjects::PersonOrCommunity(pc) => match *pc {
        UserOrCommunity::User(p) => {
          SearchableKinds::PersonOrGroup(Box::new(PersonOrGroup::Person(p.into_json(data).await?)))
        }
        UserOrCommunity::Community(c) => {
          SearchableKinds::PersonOrGroup(Box::new(PersonOrGroup::Group(c.into_json(data).await?)))
        }
      },
    })
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    apub: &Self::Kind,
    expected_domain: &Url,
    data: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    match apub {
      SearchableKinds::Page(a) => ApubPost::verify(a, expected_domain, data).await,
      SearchableKinds::Note(a) => ApubComment::verify(a, expected_domain, data).await,
      SearchableKinds::PersonOrGroup(pg) => match pg.as_ref() {
        PersonOrGroup::Person(a) => ApubPerson::verify(a, expected_domain, data).await,
        PersonOrGroup::Group(a) => ApubCommunity::verify(a, expected_domain, data).await,
      },
    }
  }

  #[tracing::instrument(skip_all)]
  async fn from_json(apub: Self::Kind, context: &Data<LemmyContext>) -> LemmyResult<Self> {
    use SearchableKinds as SAT;
    use SearchableObjects as SO;
    Ok(match apub {
      SAT::Page(p) => SO::Post(ApubPost::from_json(*p, context).await?),
      SAT::Note(n) => SO::Comment(ApubComment::from_json(n, context).await?),
      SAT::PersonOrGroup(pg) => {
        SO::PersonOrCommunity(Box::new(UserOrCommunity::from_json(*pg, context).await?))
      }
    })
  }
}
