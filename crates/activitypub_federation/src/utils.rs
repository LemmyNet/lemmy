use crate::{Error, LocalInstance, APUB_JSON_CONTENT_TYPE};
use http::StatusCode;
use serde::de::DeserializeOwned;
use tracing::log::info;
use url::Url;

pub async fn fetch_object_http<Kind: DeserializeOwned>(
  url: &Url,
  instance: &LocalInstance,
  request_counter: &mut i32,
) -> Result<Kind, Error> {
  // dont fetch local objects this way
  debug_assert!(url.domain() != Some(&instance.hostname));
  info!("Fetching remote object {}", url.to_string());

  *request_counter += 1;
  if *request_counter > instance.settings.http_fetch_retry_limit {
    return Err(Error::RequestLimit);
  }

  let res = instance
    .client
    .get(url.as_str())
    .header("Accept", APUB_JSON_CONTENT_TYPE)
    .timeout(instance.settings.request_timeout)
    .send()
    .await
    .map_err(Error::conv)?;

  if res.status() == StatusCode::GONE {
    return Err(Error::ObjectDeleted);
  }

  res.json().await.map_err(Error::conv)
}

/// Check that both urls have the same domain. If not, return UrlVerificationError.
pub fn verify_domains_match(a: &Url, b: &Url) -> Result<(), Error> {
  if a.domain() != b.domain() {
    return Err(Error::UrlVerificationError("Domains do not match"));
  }
  Ok(())
}

/// Check that both urls are identical. If not, return UrlVerificationError.
pub fn verify_urls_match(a: &Url, b: &Url) -> Result<(), Error> {
  if a != b {
    return Err(Error::UrlVerificationError("Urls do not match"));
  }
  Ok(())
}
