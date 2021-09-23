use crate::{fetcher::should_refetch_actor, objects::FromApub, APUB_JSON_CONTENT_TYPE};
use anyhow::anyhow;
use diesel::NotFound;
use lemmy_api_common::blocking;
use lemmy_db_queries::{ApubObject, DbPool};
use lemmy_db_schema::DbUrl;
use lemmy_utils::{request::retry, settings::structs::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use log::debug;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::{
  fmt::{Display, Formatter},
  marker::PhantomData,
  time::Duration,
};
use url::Url;

/// Maximum number of HTTP requests allowed to handle a single incoming activity (or a single object
/// fetch through the search). This should be configurable.
static REQUEST_LIMIT: i32 = 25;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct ObjectId<Kind>(Url, #[serde(skip)] PhantomData<Kind>)
where
  Kind: FromApub + ApubObject + Send + 'static,
  for<'de2> <Kind as FromApub>::ApubType: serde::Deserialize<'de2>;

impl<Kind> ObjectId<Kind>
where
  Kind: FromApub + ApubObject + Send + 'static,
  for<'de> <Kind as FromApub>::ApubType: serde::Deserialize<'de>,
{
  pub fn new<K, T>(url: T) -> ObjectId<K>
  where
    T: Into<Url>,
    K: FromApub + ApubObject + Send + 'static,
    for<'de> <K as FromApub>::ApubType: serde::Deserialize<'de>,
  {
    ObjectId(url.into(), PhantomData::<K>)
  }

  pub fn inner(&self) -> &Url {
    &self.0
  }

  /// Fetches an activitypub object, either from local database (if possible), or over http.
  pub(crate) async fn dereference(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<Kind, LemmyError> {
    let db_object = self.dereference_locally(context.pool()).await?;
    // if its a local object, only fetch it from the database and not over http
    if self.0.domain() == Some(&Settings::get().get_hostname_without_port()?) {
      return match db_object {
        None => Err(NotFound {}.into()),
        Some(o) => Ok(o),
      };
    }

    if let Some(object) = db_object {
      if let Some(last_refreshed_at) = object.last_refreshed_at() {
        // TODO: rename to should_refetch_object()
        if should_refetch_actor(last_refreshed_at) {
          debug!("Refetching remote object {}", self.0.as_str());
          return self.dereference_remotely(context, request_counter).await;
        }
      }
      Ok(object)
    } else {
      debug!("Fetching remote object {}", self.0.as_str());
      self.dereference_remotely(context, request_counter).await
    }
  }

  /// returning none means the object was not found in local db
  async fn dereference_locally(&self, pool: &DbPool) -> Result<Option<Kind>, LemmyError> {
    let id: DbUrl = self.0.clone().into();
    let object = blocking(pool, move |conn| ApubObject::read_from_apub_id(conn, &id)).await?;
    match object {
      Ok(o) => Ok(Some(o)),
      Err(NotFound {}) => Ok(None),
      Err(e) => Err(e.into()),
    }
  }

  async fn dereference_remotely(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<Kind, LemmyError> {
    // dont fetch local objects this way
    debug_assert!(self.0.domain() != Some(&Settings::get().hostname));

    *request_counter += 1;
    if *request_counter > REQUEST_LIMIT {
      return Err(LemmyError::from(anyhow!("Request limit reached")));
    }

    let res = retry(|| {
      context
        .client()
        .get(self.0.as_str())
        .header("Accept", APUB_JSON_CONTENT_TYPE)
        .timeout(Duration::from_secs(60))
        .send()
    })
    .await?;

    if res.status() == StatusCode::GONE {
      self.mark_object_deleted(context).await?;
      return Err(
        anyhow!(
          "Fetched remote object {} which was deleted",
          self.0.as_str()
        )
        .into(),
      );
    }

    let res2: Kind::ApubType = res.json().await?;

    Ok(Kind::from_apub(&res2, context, self.inner(), request_counter).await?)
  }

  async fn mark_object_deleted(&self, _context: &LemmyContext) -> Result<(), LemmyError> {
    // TODO: need to move methods update_deleted, update_removed etc into a trait to use them here.
    //       also, how do we know if the object was deleted vs removed?
    todo!()
  }
}

impl<Kind> Display for ObjectId<Kind>
where
  Kind: FromApub + ApubObject + Send + 'static,
  for<'de> <Kind as FromApub>::ApubType: serde::Deserialize<'de>,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0.to_string())
  }
}

impl<Kind> From<ObjectId<Kind>> for Url
where
  Kind: FromApub + ApubObject + Send + 'static,
  for<'de> <Kind as FromApub>::ApubType: serde::Deserialize<'de>,
{
  fn from(id: ObjectId<Kind>) -> Self {
    id.0
  }
}

impl<Kind> From<ObjectId<Kind>> for DbUrl
where
  Kind: FromApub + ApubObject + Send + 'static,
  for<'de> <Kind as FromApub>::ApubType: serde::Deserialize<'de>,
{
  fn from(id: ObjectId<Kind>) -> Self {
    id.0.into()
  }
}
