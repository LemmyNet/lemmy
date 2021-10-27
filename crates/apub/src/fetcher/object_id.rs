use crate::fetcher::should_refetch_object;
use anyhow::anyhow;
use diesel::NotFound;
use lemmy_apub_lib::{traits::ApubObject, APUB_JSON_CONTENT_TYPE};
use lemmy_db_schema::newtypes::DbUrl;
use lemmy_utils::{
  request::{build_user_agent, retry},
  settings::structs::Settings,
  LemmyError,
};
use log::info;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::{
  fmt::{Debug, Display, Formatter},
  marker::PhantomData,
  time::Duration,
};
use url::Url;

/// Maximum number of HTTP requests allowed to handle a single incoming activity (or a single object
/// fetch through the search). This should be configurable.
static REQUEST_LIMIT: i32 = 25;

// TODO: after moving this file to library, remove lazy_static dependency from apub crate
lazy_static! {
  static ref CLIENT: Client = Client::builder()
    .user_agent(build_user_agent(&Settings::get()))
    .build()
    .unwrap();
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct ObjectId<Kind>(Url, #[serde(skip)] PhantomData<Kind>)
where
  Kind: ApubObject + Send + 'static,
  for<'de2> <Kind as ApubObject>::ApubType: serde::Deserialize<'de2>;

impl<Kind> ObjectId<Kind>
where
  Kind: ApubObject + Send + 'static,
  for<'de2> <Kind as ApubObject>::ApubType: serde::Deserialize<'de2>,
{
  pub fn new<T>(url: T) -> Self
  where
    T: Into<Url>,
  {
    ObjectId(url.into(), PhantomData::<Kind>)
  }

  pub fn inner(&self) -> &Url {
    &self.0
  }

  /// Fetches an activitypub object, either from local database (if possible), or over http.
  pub async fn dereference(
    &self,
    data: &<Kind as ApubObject>::DataType,
    request_counter: &mut i32,
  ) -> Result<Kind, LemmyError> {
    let db_object = self.dereference_from_db(data).await?;

    // if its a local object, only fetch it from the database and not over http
    if self.0.domain() == Some(&Settings::get().get_hostname_without_port()?) {
      return match db_object {
        None => Err(NotFound {}.into()),
        Some(o) => Ok(o),
      };
    }

    // object found in database
    if let Some(object) = db_object {
      // object is old and should be refetched
      if let Some(last_refreshed_at) = object.last_refreshed_at() {
        if should_refetch_object(last_refreshed_at) {
          return self
            .dereference_from_http(data, request_counter, Some(object))
            .await;
        }
      }
      Ok(object)
    }
    // object not found, need to fetch over http
    else {
      self
        .dereference_from_http(data, request_counter, None)
        .await
    }
  }

  /// Fetch an object from the local db. Instead of falling back to http, this throws an error if
  /// the object is not found in the database.
  pub async fn dereference_local(
    &self,
    data: &<Kind as ApubObject>::DataType,
  ) -> Result<Kind, LemmyError> {
    let object = self.dereference_from_db(data).await?;
    object.ok_or_else(|| anyhow!("object not found in database {}", self).into())
  }

  /// returning none means the object was not found in local db
  async fn dereference_from_db(
    &self,
    data: &<Kind as ApubObject>::DataType,
  ) -> Result<Option<Kind>, LemmyError> {
    let id = self.0.clone();
    ApubObject::read_from_apub_id(id, data).await
  }

  async fn dereference_from_http(
    &self,
    data: &<Kind as ApubObject>::DataType,
    request_counter: &mut i32,
    db_object: Option<Kind>,
  ) -> Result<Kind, LemmyError> {
    // dont fetch local objects this way
    debug_assert!(self.0.domain() != Some(&Settings::get().hostname));
    info!("Fetching remote object {}", self.to_string());

    *request_counter += 1;
    if *request_counter > REQUEST_LIMIT {
      return Err(LemmyError::from(anyhow!("Request limit reached")));
    }

    let res = retry(|| {
      CLIENT
        .get(self.0.as_str())
        .header("Accept", APUB_JSON_CONTENT_TYPE)
        .timeout(Duration::from_secs(60))
        .send()
    })
    .await?;

    if res.status() == StatusCode::GONE {
      if let Some(db_object) = db_object {
        db_object.delete(data).await?;
      }
      return Err(anyhow!("Fetched remote object {} which was deleted", self).into());
    }

    let res2: Kind::ApubType = res.json().await?;

    Ok(Kind::from_apub(&res2, data, self.inner(), request_counter).await?)
  }
}

impl<Kind> Display for ObjectId<Kind>
where
  Kind: ApubObject + Send + 'static,
  for<'de2> <Kind as ApubObject>::ApubType: serde::Deserialize<'de2>,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0.to_string())
  }
}

impl<Kind> From<ObjectId<Kind>> for Url
where
  Kind: ApubObject + Send + 'static,
  for<'de2> <Kind as ApubObject>::ApubType: serde::Deserialize<'de2>,
{
  fn from(id: ObjectId<Kind>) -> Self {
    id.0
  }
}

impl<Kind> From<ObjectId<Kind>> for DbUrl
where
  Kind: ApubObject + Send + 'static,
  for<'de2> <Kind as ApubObject>::ApubType: serde::Deserialize<'de2>,
{
  fn from(id: ObjectId<Kind>) -> Self {
    id.0.into()
  }
}
