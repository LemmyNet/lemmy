// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::APUB_JSON_CONTENT_TYPE;
use anyhow::anyhow;
use http::StatusCode;
use lemmy_utils::{request::retry, settings::structs::Settings, LemmyError, REQWEST_TIMEOUT};
use reqwest_middleware::ClientWithMiddleware;
use serde::de::DeserializeOwned;
use tracing::log::info;
use url::Url;

pub async fn fetch_object_http<Kind: DeserializeOwned>(
  url: &Url,
  client: &ClientWithMiddleware,
  request_counter: &mut i32,
) -> Result<Kind, LemmyError> {
  // dont fetch local objects this way
  debug_assert!(url.domain() != Some(&Settings::get().hostname));
  info!("Fetching remote object {}", url.to_string());

  *request_counter += 1;
  if *request_counter > Settings::get().http_fetch_retry_limit {
    return Err(LemmyError::from(anyhow!("Request retry limit reached")));
  }

  let res = retry(|| {
    client
      .get(url.as_str())
      .header("Accept", APUB_JSON_CONTENT_TYPE)
      .timeout(REQWEST_TIMEOUT)
      .send()
  })
  .await?;

  if res.status() == StatusCode::GONE {
    return Err(LemmyError::from_message("410"));
  }

  Ok(res.json().await?)
}
