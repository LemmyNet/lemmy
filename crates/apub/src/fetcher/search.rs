use super::post_or_comment::{PageOrNote, PostOrComment};
use crate::fetcher::user_or_community::{PersonOrGroup, UserOrCommunity};
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
  PostOrComment(Box<PostOrComment>),
  PersonOrCommunity(Box<UserOrCommunity>),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum SearchableKinds {
  PageOrNote(Box<PageOrNote>),
  PersonOrGroup(Box<PersonOrGroup>),
}

#[async_trait::async_trait]
impl Object for SearchableObjects {
  type DataType = LemmyContext;
  type Kind = SearchableKinds;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    match self {
      SearchableObjects::PostOrComment(p) => p.last_refreshed_at(),
      SearchableObjects::PersonOrCommunity(p) => p.last_refreshed_at(),
    }
  }

  // TODO: this is inefficient, because if the object is not in local db, it will run 4 db queries
  //       before finally returning an error. it would be nice if we could check all 4 tables in
  //       a single query.
  //       we could skip this and always return an error, but then it would always fetch objects
  //       over http, and not be able to mark objects as deleted that were deleted by remote server.
  async fn read_from_id(
    object_id: Url,
    context: &Data<Self::DataType>,
  ) -> LemmyResult<Option<Self>> {
    let uc = UserOrCommunity::read_from_id(object_id.clone(), context).await?;
    if let Some(uc) = uc {
      return Ok(Some(SearchableObjects::PersonOrCommunity(Box::new(uc))));
    }
    let pc = PostOrComment::read_from_id(object_id.clone(), context).await?;
    if let Some(pc) = pc {
      return Ok(Some(SearchableObjects::PostOrComment(Box::new(pc))));
    }
    Ok(None)
  }

  async fn delete(self, data: &Data<Self::DataType>) -> LemmyResult<()> {
    match self {
      SearchableObjects::PostOrComment(pc) => pc.delete(data).await,
      SearchableObjects::PersonOrCommunity(pc) => pc.delete(data).await,
    }
  }

  async fn into_json(self, data: &Data<Self::DataType>) -> LemmyResult<Self::Kind> {
    use SearchableObjects::*;
    Ok(match self {
      PostOrComment(pc) => SearchableKinds::PageOrNote(Box::new(pc.into_json(data).await?)),
      PersonOrCommunity(pc) => SearchableKinds::PersonOrGroup(Box::new(pc.into_json(data).await?)),
    })
  }

  async fn verify(
    apub: &Self::Kind,
    expected_domain: &Url,
    data: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    use SearchableKinds::*;
    match apub {
      PageOrNote(pn) => PostOrComment::verify(pn, expected_domain, data).await,
      PersonOrGroup(pg) => UserOrCommunity::verify(pg, expected_domain, data).await,
    }
  }

  async fn from_json(apub: Self::Kind, context: &Data<LemmyContext>) -> LemmyResult<Self> {
    use SearchableKinds::*;
    use SearchableObjects as SO;
    Ok(match apub {
      PageOrNote(pg) => SO::PostOrComment(Box::new(PostOrComment::from_json(*pg, context).await?)),
      PersonOrGroup(pg) => {
        SO::PersonOrCommunity(Box::new(UserOrCommunity::from_json(*pg, context).await?))
      }
    })
  }
}
