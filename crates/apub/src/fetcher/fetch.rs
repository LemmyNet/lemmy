use crate::{check_is_apub_id_valid, APUB_JSON_CONTENT_TYPE};
use anyhow::anyhow;
use lemmy_utils::{request::retry, LemmyError};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;
use url::Url;

/// Maximum number of HTTP requests allowed to handle a single incoming activity (or a single object
/// fetch through the search).
///
/// A community fetch will load the outbox with up to 20 items, and fetch the creator for each item.
/// So we are looking at a maximum of 22 requests (rounded up just to be safe).
static MAX_REQUEST_NUMBER: i32 = 25;

#[derive(Debug, Error)]
pub(in crate::fetcher) struct FetchError {
  pub inner: anyhow::Error,
  pub status_code: Option<StatusCode>,
}

impl From<LemmyError> for FetchError {
  fn from(t: LemmyError) -> Self {
    FetchError {
      inner: t.inner,
      status_code: None,
    }
  }
}

impl From<reqwest::Error> for FetchError {
  fn from(t: reqwest::Error) -> Self {
    let status = t.status();
    FetchError {
      inner: t.into(),
      status_code: status,
    }
  }
}

impl std::fmt::Display for FetchError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    std::fmt::Display::fmt(&self, f)
  }
}

/// Fetch any type of ActivityPub object, handling things like HTTP headers, deserialisation,
/// timeouts etc.
pub(in crate::fetcher) async fn fetch_remote_object<Response>(
  client: &Client,
  url: &Url,
  recursion_counter: &mut i32,
) -> Result<Response, FetchError>
where
  Response: for<'de> Deserialize<'de> + std::fmt::Debug,
{
  *recursion_counter += 1;
  if *recursion_counter > MAX_REQUEST_NUMBER {
    return Err(LemmyError::from(anyhow!("Maximum recursion depth reached")).into());
  }
  check_is_apub_id_valid(&url)?;

  let timeout = Duration::from_secs(60);

  let res = retry(|| {
    client
      .get(url.as_str())
      .header("Accept", APUB_JSON_CONTENT_TYPE)
      .timeout(timeout)
      .send()
  })
  .await?;

  if res.status() == StatusCode::GONE {
    return Err(FetchError {
      inner: anyhow!("Remote object {} was deleted", url),
      status_code: Some(res.status()),
    });
  }

  Ok(res.json().await?)
}
