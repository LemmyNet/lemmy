use crate::{settings::structs::Settings, LemmyError};
use anyhow::anyhow;
use log::error;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Client;
use serde::Deserialize;
use std::future::Future;
use thiserror::Error;
use url::Url;

#[derive(Clone, Debug, Error)]
#[error("Error sending request, {0}")]
struct SendError(pub String);

#[derive(Clone, Debug, Error)]
#[error("Error receiving response, {0}")]
pub struct RecvError(pub String);

pub async fn retry<F, Fut, T>(f: F) -> Result<T, reqwest::Error>
where
  F: Fn() -> Fut,
  Fut: Future<Output = Result<T, reqwest::Error>>,
{
  retry_custom(|| async { Ok((f)().await) }).await
}

async fn retry_custom<F, Fut, T>(f: F) -> Result<T, reqwest::Error>
where
  F: Fn() -> Fut,
  Fut: Future<Output = Result<Result<T, reqwest::Error>, reqwest::Error>>,
{
  let mut response: Option<Result<T, reqwest::Error>> = None;

  for _ in 0u8..3 {
    match (f)().await? {
      Ok(t) => return Ok(t),
      Err(e) => {
        if e.is_timeout() {
          response = Some(Err(e));
          continue;
        }
        return Err(e);
      }
    }
  }

  response.expect("retry http request")
}

#[derive(Deserialize, Debug)]
pub struct IframelyResponse {
  pub title: Option<String>,
  pub description: Option<String>,
  thumbnail_url: Option<Url>,
  pub html: Option<String>,
}

pub(crate) async fn fetch_iframely(
  client: &Client,
  url: &Url,
) -> Result<IframelyResponse, LemmyError> {
  if let Some(iframely_url) = Settings::get().iframely_url {
    let fetch_url = format!("{}/oembed?url={}", iframely_url, url);

    let response = retry(|| client.get(&fetch_url).send()).await?;

    let res: IframelyResponse = response
      .json()
      .await
      .map_err(|e| RecvError(e.to_string()))?;
    Ok(res)
  } else {
    Err(anyhow!("Missing Iframely URL in config.").into())
  }
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct PictrsResponse {
  files: Vec<PictrsFile>,
  msg: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct PictrsFile {
  file: String,
  delete_token: String,
}

pub(crate) async fn fetch_pictrs(
  client: &Client,
  image_url: &Url,
) -> Result<Option<PictrsResponse>, LemmyError> {
  if let Some(pictrs_url) = Settings::get().pictrs_url {
    is_image_content_type(client, image_url).await?;

    let fetch_url = format!(
      "{}/image/download?url={}",
      pictrs_url,
      utf8_percent_encode(image_url.as_str(), NON_ALPHANUMERIC) // TODO this might not be needed
    );

    let response = retry(|| client.get(&fetch_url).send()).await?;

    let response: PictrsResponse = response
      .json()
      .await
      .map_err(|e| RecvError(e.to_string()))?;

    if response.msg == "ok" {
      Ok(Some(response))
    } else {
      Err(anyhow!("{}", &response.msg).into())
    }
  } else {
    Ok(None)
  }
}

pub async fn fetch_iframely_and_pictrs_data(
  client: &Client,
  url: Option<&Url>,
) -> Result<(Option<IframelyResponse>, Option<Url>), LemmyError> {
  match &url {
    Some(url) => {
      // Fetch iframely data
      let iframely_res_option = fetch_iframely(client, url).await.ok();

      // Fetch pictrs thumbnail
      let pictrs_hash = match &iframely_res_option {
        Some(iframely_res) => match &iframely_res.thumbnail_url {
          Some(iframely_thumbnail_url) => fetch_pictrs(client, iframely_thumbnail_url)
            .await?
            .map(|r| r.files[0].file.to_owned()),
          // Try to generate a small thumbnail if iframely is not supported
          None => fetch_pictrs(client, url)
            .await?
            .map(|r| r.files[0].file.to_owned()),
        },
        None => fetch_pictrs(client, url)
          .await?
          .map(|r| r.files[0].file.to_owned()),
      };

      // The full urls are necessary for federation
      let pictrs_thumbnail = pictrs_hash
        .map(|p| {
          Url::parse(&format!(
            "{}/pictrs/image/{}",
            Settings::get().get_protocol_and_hostname(),
            p
          ))
          .ok()
        })
        .flatten();

      Ok((iframely_res_option, pictrs_thumbnail))
    }
    None => Ok((None, None)),
  }
}

async fn is_image_content_type(client: &Client, test: &Url) -> Result<(), LemmyError> {
  let response = retry(|| client.get(test.to_owned()).send()).await?;
  if response
    .headers()
    .get("Content-Type")
    .ok_or_else(|| anyhow!("No Content-Type header"))?
    .to_str()?
    .starts_with("image/")
  {
    Ok(())
  } else {
    Err(anyhow!("Not an image type.").into())
  }
}

#[cfg(test)]
mod tests {
  // These helped with testing
  // #[test]
  // fn test_iframely() {
  //   let res = fetch_iframely(client, "https://www.redspark.nu/?p=15341").await;
  //   assert!(res.is_ok());
  // }

  // #[test]
  // fn test_pictshare() {
  //   let res = fetch_pictshare("https://upload.wikimedia.org/wikipedia/en/2/27/The_Mandalorian_logo.jpg");
  //   assert!(res.is_ok());
  //   let res_other = fetch_pictshare("https://upload.wikimedia.org/wikipedia/en/2/27/The_Mandalorian_logo.jpgaoeu");
  //   assert!(res_other.is_err());
  // }
}
