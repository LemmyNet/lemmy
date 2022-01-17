use crate::{traits::ApubObject, APUB_JSON_CONTENT_TYPE};
use anyhow::anyhow;
use chrono::{Duration as ChronoDuration, NaiveDateTime, Utc};
use diesel::NotFound;
use lemmy_utils::{request::retry, settings::structs::Settings, LemmyError};
use reqwest::StatusCode;
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};
use std::{
  fmt::{Debug, Display, Formatter},
  marker::PhantomData,
  time::Duration,
};
use tracing::info;
use url::Url;

/// We store Url on the heap because it is quite large (88 bytes).
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct ObjectId<Kind>(Box<Url>, #[serde(skip)] PhantomData<Kind>)
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
    ObjectId(Box::new(url.into()), PhantomData::<Kind>)
  }

  pub fn inner(&self) -> &Url {
    &self.0
  }

  /// Fetches an activitypub object, either from local database (if possible), or over http.
  pub async fn dereference(
    &self,
    data: &<Kind as ApubObject>::DataType,
    client: &ClientWithMiddleware,
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
            .dereference_from_http(data, client, request_counter, Some(object))
            .await;
        }
      }
      Ok(object)
    }
    // object not found, need to fetch over http
    else {
      self
        .dereference_from_http(data, client, request_counter, None)
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
    ApubObject::read_from_apub_id(*id, data).await
  }

  async fn dereference_from_http(
    &self,
    data: &<Kind as ApubObject>::DataType,
    client: &ClientWithMiddleware,
    request_counter: &mut i32,
    db_object: Option<Kind>,
  ) -> Result<Kind, LemmyError> {
    // dont fetch local objects this way
    debug_assert!(self.0.domain() != Some(&Settings::get().hostname));
    info!("Fetching remote object {}", self.to_string());

    *request_counter += 1;
    if *request_counter > Settings::get().http_fetch_retry_limit {
      return Err(LemmyError::from(anyhow!("Request retry limit reached")));
    }

    let res = retry(|| {
      client
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

    Kind::verify(&res2, self.inner(), data, request_counter).await?;
    Ok(Kind::from_apub(res2, data, request_counter).await?)
  }
}

static ACTOR_REFETCH_INTERVAL_SECONDS: i64 = 24 * 60 * 60;
static ACTOR_REFETCH_INTERVAL_SECONDS_DEBUG: i64 = 10;

/// Determines when a remote actor should be refetched from its instance. In release builds, this is
/// `ACTOR_REFETCH_INTERVAL_SECONDS` after the last refetch, in debug builds
/// `ACTOR_REFETCH_INTERVAL_SECONDS_DEBUG`.
///
/// TODO it won't pick up new avatars, summaries etc until a day after.
/// Actors need an "update" activity pushed to other servers to fix this.
fn should_refetch_object(last_refreshed: NaiveDateTime) -> bool {
  let update_interval = if cfg!(debug_assertions) {
    // avoid infinite loop when fetching community outbox
    ChronoDuration::seconds(ACTOR_REFETCH_INTERVAL_SECONDS_DEBUG)
  } else {
    ChronoDuration::seconds(ACTOR_REFETCH_INTERVAL_SECONDS)
  };
  let refresh_limit = Utc::now().naive_utc() - update_interval;
  last_refreshed.lt(&refresh_limit)
}

impl<Kind> Display for ObjectId<Kind>
where
  Kind: ApubObject + Send + 'static,
  for<'de2> <Kind as ApubObject>::ApubType: serde::Deserialize<'de2>,
{
  #[allow(clippy::to_string_in_display)]
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    // Use to_string here because Url.display is not useful for us
    write!(f, "{}", self.0)
  }
}

impl<Kind> From<ObjectId<Kind>> for Url
where
  Kind: ApubObject + Send + 'static,
  for<'de2> <Kind as ApubObject>::ApubType: serde::Deserialize<'de2>,
{
  fn from(id: ObjectId<Kind>) -> Self {
    *id.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::object_id::should_refetch_object;

  #[test]
  fn test_should_refetch_object() {
    let one_second_ago = Utc::now().naive_utc() - ChronoDuration::seconds(1);
    assert!(!should_refetch_object(one_second_ago));

    let two_days_ago = Utc::now().naive_utc() - ChronoDuration::days(2);
    assert!(should_refetch_object(two_days_ago));
  }
}
