use crate::{settings::structs::Settings, version::VERSION, LemmyError};
use anyhow::anyhow;
use encoding::{all::encodings, DecoderTrap};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};
use std::future::Future;
use thiserror::Error;
use tracing::{error, info};
use url::Url;
use webpage::HTML;

#[derive(Clone, Debug, Error)]
#[error("Error sending request, {0}")]
struct SendError(pub String);

#[derive(Clone, Debug, Error)]
#[error("Error receiving response, {0}")]
pub struct RecvError(pub String);

pub async fn retry<F, Fut, T>(f: F) -> Result<T, reqwest_middleware::Error>
where
  F: Fn() -> Fut,
  Fut: Future<Output = Result<T, reqwest_middleware::Error>>,
{
  retry_custom(|| async { Ok((f)().await) }).await
}

async fn retry_custom<F, Fut, T>(f: F) -> Result<T, reqwest_middleware::Error>
where
  F: Fn() -> Fut,
  Fut: Future<Output = Result<Result<T, reqwest_middleware::Error>, reqwest_middleware::Error>>,
{
  let mut response: Option<Result<T, reqwest_middleware::Error>> = None;

  for _ in 0u8..3 {
    match (f)().await? {
      Ok(t) => return Ok(t),
      Err(reqwest_middleware::Error::Reqwest(e)) => {
        if e.is_timeout() {
          response = Some(Err(reqwest_middleware::Error::Reqwest(e)));
          continue;
        }
        return Err(reqwest_middleware::Error::Reqwest(e));
      }
      Err(otherwise) => {
        return Err(otherwise);
      }
    }
  }

  response.expect("retry http request")
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct SiteMetadata {
  pub title: Option<String>,
  pub description: Option<String>,
  image: Option<Url>,
  pub html: Option<String>,
}

/// Fetches the post link html tags (like title, description, image, etc)
pub async fn fetch_site_metadata(
  client: &ClientWithMiddleware,
  url: &Url,
) -> Result<SiteMetadata, LemmyError> {
  info!("Fetching site metadata for url: {}", url);
  let response = client.get(url.as_str()).send().await?;

  // Can't use .text() here, because it only checks the content header, not the actual bytes
  // https://github.com/LemmyNet/lemmy/issues/1964
  let html_bytes = response
    .bytes()
    .await
    .map_err(|e| RecvError(e.to_string()))?
    .to_vec();

  let tags = html_to_site_metadata(&html_bytes)?;

  Ok(tags)
}

fn html_to_site_metadata(html_bytes: &[u8]) -> Result<SiteMetadata, LemmyError> {
  let html = String::from_utf8_lossy(html_bytes);

  // Make sure the first line is doctype html
  let first_line = html
    .trim_start()
    .lines()
    .into_iter()
    .next()
    .ok_or_else(|| LemmyError::from_message("No lines in html"))?
    .to_lowercase();

  if !first_line.starts_with("<!doctype html>") {
    return Err(LemmyError::from_message(
      "Site metadata page fetch is not DOCTYPE html",
    ));
  }

  let mut page = HTML::from_string(html.to_string(), None)?;

  // If the web page specifies that it isn't actually UTF-8, re-decode the received bytes with the
  // proper encoding. If the specified encoding cannot be found, fall back to the original UTF-8
  // version.
  if let Some(charset) = page.meta.get("charset") {
    if charset.to_lowercase() != "utf-8" {
      if let Some(encoding_ref) = encodings().iter().find(|e| e.name() == charset) {
        if let Ok(html_with_encoding) = encoding_ref.decode(html_bytes, DecoderTrap::Replace) {
          page = HTML::from_string(html_with_encoding, None)?;
        }
      }
    }
  }

  let page_title = page.title;
  let page_description = page.description;

  let og_description = page
    .opengraph
    .properties
    .get("description")
    .map(|t| t.to_string());
  let og_title = page
    .opengraph
    .properties
    .get("title")
    .map(|t| t.to_string());
  let og_image = page
    .opengraph
    .images
    .get(0)
    .map(|ogo| Url::parse(&ogo.url).ok())
    .flatten();

  let title = og_title.or(page_title);
  let description = og_description.or(page_description);
  let image = og_image;

  Ok(SiteMetadata {
    title,
    description,
    image,
    html: None,
  })
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct PictrsResponse {
  files: Vec<PictrsFile>,
  msg: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct PictrsFile {
  file: String,
  #[allow(dead_code)]
  delete_token: String,
}

pub(crate) async fn fetch_pictrs(
  client: &ClientWithMiddleware,
  settings: &Settings,
  image_url: &Url,
) -> Result<PictrsResponse, LemmyError> {
  if let Some(pictrs_url) = settings.pictrs_url.to_owned() {
    is_image_content_type(client, image_url).await?;

    let fetch_url = format!(
      "{}/image/download?url={}",
      pictrs_url,
      utf8_percent_encode(image_url.as_str(), NON_ALPHANUMERIC) // TODO this might not be needed
    );

    let response = client.get(&fetch_url).send().await?;

    let response: PictrsResponse = response
      .json()
      .await
      .map_err(|e| RecvError(e.to_string()))?;

    if response.msg == "ok" {
      Ok(response)
    } else {
      Err(anyhow!("{}", &response.msg).into())
    }
  } else {
    Err(anyhow!("pictrs_url not set up in config").into())
  }
}

/// Both are options, since the URL might be either an html page, or an image
/// Returns the SiteMetadata, and a Pictrs URL, if there is a picture associated
pub async fn fetch_site_data(
  client: &ClientWithMiddleware,
  settings: &Settings,
  url: Option<&Url>,
) -> (Option<SiteMetadata>, Option<Url>) {
  match &url {
    Some(url) => {
      // Fetch metadata
      // Ignore errors, since it may be an image, or not have the data.
      // Warning, this may ignore SSL errors
      let metadata_option = fetch_site_metadata(client, url).await.ok();

      // Fetch pictrs thumbnail
      let pictrs_hash = match &metadata_option {
        Some(metadata_res) => match &metadata_res.image {
          // Metadata, with image
          // Try to generate a small thumbnail if there's a full sized one from post-links
          Some(metadata_image) => fetch_pictrs(client, settings, metadata_image)
            .await
            .map(|r| r.files[0].file.to_owned()),
          // Metadata, but no image
          None => fetch_pictrs(client, settings, url)
            .await
            .map(|r| r.files[0].file.to_owned()),
        },
        // No metadata, try to fetch the URL as an image
        None => fetch_pictrs(client, settings, url)
          .await
          .map(|r| r.files[0].file.to_owned()),
      };

      // The full urls are necessary for federation
      let pictrs_thumbnail = pictrs_hash
        .map(|p| {
          Url::parse(&format!(
            "{}/pictrs/image/{}",
            settings.get_protocol_and_hostname(),
            p
          ))
          .ok()
        })
        .ok()
        .flatten();

      (metadata_option, pictrs_thumbnail)
    }
    None => (None, None),
  }
}

async fn is_image_content_type(client: &ClientWithMiddleware, url: &Url) -> Result<(), LemmyError> {
  let response = client.get(url.as_str()).send().await?;
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

pub fn build_user_agent(settings: &Settings) -> String {
  format!(
    "Lemmy/{}; +{}",
    VERSION,
    settings.get_protocol_and_hostname()
  )
}

#[cfg(test)]
mod tests {
  use crate::request::{build_user_agent, fetch_site_metadata};
  use url::Url;

  use super::SiteMetadata;
  use crate::settings::structs::Settings;

  // These helped with testing
  #[actix_rt::test]
  async fn test_site_metadata() {
    let settings = Settings::init().unwrap();
    let client = reqwest::Client::builder()
      .user_agent(build_user_agent(&settings))
      .build()
      .unwrap()
      .into();
    let sample_url = Url::parse("https://www.redspark.nu/en/peoples-war/district-leader-of-chand-led-cpn-arrested-in-bhojpur/").unwrap();
    let sample_res = fetch_site_metadata(&client, &sample_url).await.unwrap();
    assert_eq!(
      SiteMetadata {
        title: Some("District Leader Of Chand Led CPN Arrested In Bhojpur - Redspark".to_string()),
        description: Some("BHOJPUR: A district leader of the outlawed Netra Bikram Chand alias Biplav-led outfit has been arrested. According to District Police".to_string()),
        image: Some(Url::parse("https://www.redspark.nu/wp-content/uploads/2020/03/netra-bikram-chand-attends-program-1272019033653-1000x0-845x653-1.jpg").unwrap()),
        html: None,
      }, sample_res);

    let youtube_url = Url::parse("https://www.youtube.com/watch?v=IquO_TcMZIQ").unwrap();
    let youtube_res = fetch_site_metadata(&client, &youtube_url).await.unwrap();
    assert_eq!(
      SiteMetadata {
        title: Some("A Hard Look at Rent and Rent Seeking with Michael Hudson & Pepe Escobar".to_string()),
        description: Some("An interactive discussion on wealth inequality and the “Great Game” on the control of natural resources.In this webinar organized jointly by the Henry George...".to_string()),
        image: Some(Url::parse("https://i.ytimg.com/vi/IquO_TcMZIQ/maxresdefault.jpg").unwrap()),
        html: None,
      }, youtube_res);
  }

  // #[test]
  // fn test_pictshare() {
  //   let res = fetch_pictshare("https://upload.wikimedia.org/wikipedia/en/2/27/The_Mandalorian_logo.jpg");
  //   assert!(res.is_ok());
  //   let res_other = fetch_pictshare("https://upload.wikimedia.org/wikipedia/en/2/27/The_Mandalorian_logo.jpgaoeu");
  //   assert!(res_other.is_err());
  // }
}
