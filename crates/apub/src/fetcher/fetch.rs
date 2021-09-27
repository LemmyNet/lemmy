use crate::{check_is_apub_id_valid, APUB_JSON_CONTENT_TYPE};
use anyhow::anyhow;
use lemmy_utils::{request::retry, settings::structs::Settings, LemmyError};
use log::info;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use url::Url;

/// Maximum number of HTTP requests allowed to handle a single incoming activity (or a single object
/// fetch through the search).
///
/// A community fetch will load the outbox with up to 20 items, and fetch the creator for each item.
/// So we are looking at a maximum of 22 requests (rounded up just to be safe).
static MAX_REQUEST_NUMBER: i32 = 25;

/// Fetch any type of ActivityPub object, handling things like HTTP headers, deserialisation,
/// timeouts etc.
pub(in crate::fetcher) async fn fetch_remote_object<Response>(
  client: &Client,
  settings: &Settings,
  url: &Url,
  recursion_counter: &mut i32,
) -> Result<Response, LemmyError>
where
  Response: for<'de> Deserialize<'de> + std::fmt::Debug,
{
  *recursion_counter += 1;
  if *recursion_counter > MAX_REQUEST_NUMBER {
    return Err(anyhow!("Maximum recursion depth reached").into());
  }
  check_is_apub_id_valid(url, false, settings)?;

  let timeout = Duration::from_secs(60);

  let res = retry(|| {
    client
      .get(url.as_str())
      .header("Accept", APUB_JSON_CONTENT_TYPE)
      .timeout(timeout)
      .send()
  })
  .await?;

  let object = res.json().await?;
  info!("Fetched remote object {}", url);
  Ok(object)
}
