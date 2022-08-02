use crate::post::SiteMetadata;
use encoding::{all::encodings, DecoderTrap};
use lemmy_db_schema::newtypes::DbUrl;
use lemmy_utils::{
  error::LemmyError,
  settings::structs::Settings,
  version::VERSION,
  REQWEST_TIMEOUT,
};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;
use tracing::info;
use url::Url;
use webpage::HTML;

/// Fetches the post link html tags (like title, description, image, etc)
#[tracing::instrument(skip_all)]
pub async fn fetch_site_metadata(
  client: &ClientWithMiddleware,
  url: &Url,
) -> Result<SiteMetadata, LemmyError> {
  info!("Fetching site metadata for url: {}", url);
  let response = client.get(url.as_str()).send().await?;

  // Can't use .text() here, because it only checks the content header, not the actual bytes
  // https://github.com/LemmyNet/lemmy/issues/1964
  let html_bytes = response.bytes().await.map_err(LemmyError::from)?.to_vec();

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
    .and_then(|ogo| Url::parse(&ogo.url).ok());
  let og_embed_url = page
    .opengraph
    .videos
    .first()
    .and_then(|v| Url::parse(&v.url).ok());

  Ok(SiteMetadata {
    title: og_title.or(page_title),
    description: og_description.or(page_description),
    image: og_image.map(Into::into),
    embed_video_url: og_embed_url.map(Into::into),
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

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct PictrsPurgeResponse {
  msg: String,
}

#[tracing::instrument(skip_all)]
pub(crate) async fn fetch_pictrs(
  client: &ClientWithMiddleware,
  settings: &Settings,
  image_url: &Url,
) -> Result<PictrsResponse, LemmyError> {
  let pictrs_config = settings.pictrs_config()?;
  is_image_content_type(client, image_url).await?;

  let fetch_url = format!(
    "{}image/download?url={}",
    pictrs_config.url,
    utf8_percent_encode(image_url.as_str(), NON_ALPHANUMERIC) // TODO this might not be needed
  );

  let response = client
    .get(&fetch_url)
    .timeout(REQWEST_TIMEOUT)
    .send()
    .await?;

  let response: PictrsResponse = response.json().await.map_err(LemmyError::from)?;

  if response.msg == "ok" {
    Ok(response)
  } else {
    Err(LemmyError::from_message(&response.msg))
  }
}

/// Purges an image from pictrs
/// Note: This should often be coerced from a Result to .ok() in order to fail softly, because:
/// - It might fail due to image being not local
/// - It might not be an image
/// - Pictrs might not be set up
pub async fn purge_image_from_pictrs(
  client: &ClientWithMiddleware,
  settings: &Settings,
  image_url: &Url,
) -> Result<(), LemmyError> {
  let pictrs_config = settings.pictrs_config()?;
  is_image_content_type(client, image_url).await?;

  let alias = image_url
    .path_segments()
    .ok_or_else(|| LemmyError::from_message("Image URL missing path segments"))?
    .next_back()
    .ok_or_else(|| LemmyError::from_message("Image URL missing last path segment"))?;

  let purge_url = format!("{}/internal/purge?alias={}", pictrs_config.url, alias);

  let pictrs_api_key = pictrs_config
    .api_key
    .ok_or_else(|| LemmyError::from_message("pictrs_api_key_not_provided"))?;
  let response = client
    .post(&purge_url)
    .timeout(REQWEST_TIMEOUT)
    .header("x-api-token", pictrs_api_key)
    .send()
    .await?;

  let response: PictrsPurgeResponse = response.json().await.map_err(LemmyError::from)?;

  if response.msg == "ok" {
    Ok(())
  } else {
    Err(LemmyError::from_message(&response.msg))
  }
}

/// Both are options, since the URL might be either an html page, or an image
/// Returns the SiteMetadata, and a Pictrs URL, if there is a picture associated
#[tracing::instrument(skip_all)]
pub async fn fetch_site_data(
  client: &ClientWithMiddleware,
  settings: &Settings,
  url: Option<&Url>,
) -> (Option<SiteMetadata>, Option<DbUrl>) {
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

      (metadata_option, pictrs_thumbnail.map(Into::into))
    }
    None => (None, None),
  }
}

#[tracing::instrument(skip_all)]
async fn is_image_content_type(client: &ClientWithMiddleware, url: &Url) -> Result<(), LemmyError> {
  let response = client.get(url.as_str()).send().await?;
  if response
    .headers()
    .get("Content-Type")
    .ok_or_else(|| LemmyError::from_message("No Content-Type header"))?
    .to_str()?
    .starts_with("image/")
  {
    Ok(())
  } else {
    Err(LemmyError::from_message("Not an image type."))
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
  use crate::request::{build_user_agent, fetch_site_metadata, SiteMetadata};
  use lemmy_utils::settings::SETTINGS;
  use url::Url;

  // These helped with testing
  #[actix_rt::test]
  async fn test_site_metadata() {
    let settings = &SETTINGS.to_owned();
    let client = reqwest::Client::builder()
      .user_agent(build_user_agent(settings))
      .build()
      .unwrap()
      .into();
    let sample_url = Url::parse("https://gitlab.com/IzzyOnDroid/repo/-/wikis/FAQ").unwrap();
    let sample_res = fetch_site_metadata(&client, &sample_url).await.unwrap();
    assert_eq!(
      SiteMetadata {
        title: Some("FAQ · Wiki · IzzyOnDroid / repo · GitLab".to_string()),
        description: Some(
          "The F-Droid compatible repo at https://apt.izzysoft.de/fdroid/".to_string()
        ),
        image: Some(
          Url::parse("https://gitlab.com/uploads/-/system/project/avatar/4877469/iod_logo.png")
            .unwrap()
            .into()
        ),
        embed_video_url: None,
      },
      sample_res
    );
  }

  // #[test]
  // fn test_pictshare() {
  //   let res = fetch_pictshare("https://upload.wikimedia.org/wikipedia/en/2/27/The_Mandalorian_logo.jpg");
  //   assert!(res.is_ok());
  //   let res_other = fetch_pictshare("https://upload.wikimedia.org/wikipedia/en/2/27/The_Mandalorian_logo.jpgaoeu");
  //   assert!(res_other.is_err());
  // }
}
