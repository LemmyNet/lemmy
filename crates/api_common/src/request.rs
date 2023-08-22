use crate::post::SiteMetadata;
use encoding::{all::encodings, DecoderTrap};
use lemmy_db_schema::newtypes::DbUrl;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorType},
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

  let tags = html_to_site_metadata(&html_bytes, url)?;

  Ok(tags)
}

fn html_to_site_metadata(html_bytes: &[u8], url: &Url) -> Result<SiteMetadata, LemmyError> {
  let html = String::from_utf8_lossy(html_bytes);

  // Make sure the first line is doctype html
  let first_line = html
    .trim_start()
    .lines()
    .next()
    .ok_or(LemmyErrorType::NoLinesInHtml)?
    .to_lowercase();

  if !first_line.starts_with("<!doctype html>") {
    Err(LemmyErrorType::SiteMetadataPageIsNotDoctypeHtml)?;
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
    .map(std::string::ToString::to_string);
  let og_title = page
    .opengraph
    .properties
    .get("title")
    .map(std::string::ToString::to_string);
  let og_image = page
    .opengraph
    .images
    .first()
    // join also works if the target URL is absolute
    .and_then(|ogo| url.join(&ogo.url).ok());
  let og_embed_url = page
    .opengraph
    .videos
    .first()
    // join also works if the target URL is absolute
    .and_then(|v| url.join(&v.url).ok());

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

  // fetch remote non-pictrs images for persistent thumbnail link
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
    Err(LemmyErrorType::PictrsResponseError(response.msg))?
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
    .ok_or(LemmyErrorType::ImageUrlMissingPathSegments)?
    .next_back()
    .ok_or(LemmyErrorType::ImageUrlMissingLastPathSegment)?;

  let purge_url = format!("{}/internal/purge?alias={}", pictrs_config.url, alias);

  let pictrs_api_key = pictrs_config
    .api_key
    .ok_or(LemmyErrorType::PictrsApiKeyNotProvided)?;
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
    Err(LemmyErrorType::PictrsPurgeResponseError(response.msg))?
  }
}

/// Both are options, since the URL might be either an html page, or an image
/// Returns the SiteMetadata, and a Pictrs URL, if there is a picture associated
#[tracing::instrument(skip_all)]
pub async fn fetch_site_data(
  client: &ClientWithMiddleware,
  settings: &Settings,
  url: Option<&Url>,
  include_image: bool,
) -> (Option<SiteMetadata>, Option<DbUrl>) {
  match &url {
    Some(url) => {
      // Fetch metadata
      // Ignore errors, since it may be an image, or not have the data.
      // Warning, this may ignore SSL errors
      let metadata_option = fetch_site_metadata(client, url).await.ok();
      if !include_image {
        return (metadata_option, None);
      }

      let missing_pictrs_file =
        |r: PictrsResponse| r.files.first().expect("missing pictrs file").file.clone();

      let cache_remote_images = settings
        .pictrs_config()
        .map(|config| config.cache_remote_images)
        .unwrap_or(true);
      if !cache_remote_images {
        return match is_image_content_type(client, url).await {
          Ok(_) => {
            let url = <Url>::clone(url);
            let url = metadata_option
              .clone()
              .and_then(|metadata| metadata.image)
              .or_else(|| Some(url.into()));
            (metadata_option, url)
          }
          Err(_) => (metadata_option, None),
        };
      }

      // Fetch pictrs thumbnail
      let pictrs_hash = match &metadata_option {
        Some(metadata_res) => match &metadata_res.image {
          // Metadata, with image
          // Try to generate a small thumbnail if there's a full sized one from post-links
          Some(metadata_image) => fetch_pictrs(client, settings, metadata_image)
            .await
            .map(missing_pictrs_file),
          // Metadata, but no image
          None => fetch_pictrs(client, settings, url)
            .await
            .map(missing_pictrs_file),
        },
        // No metadata, try to fetch the URL as an image
        None => fetch_pictrs(client, settings, url)
          .await
          .map(missing_pictrs_file),
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
    .ok_or(LemmyErrorType::NoContentTypeHeader)?
    .to_str()?
    .starts_with("image/")
  {
    Ok(())
  } else {
    Err(LemmyErrorType::NotAnImageType)?
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
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::request::{
    build_user_agent,
    fetch_site_metadata,
    html_to_site_metadata,
    SiteMetadata,
  };
  use lemmy_utils::settings::SETTINGS;
  use url::Url;

  // These helped with testing
  #[tokio::test]
  async fn test_site_metadata() {
    let settings = &SETTINGS.clone();
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

  #[test]
  fn test_resolve_image_url() {
    // url that lists the opengraph fields
    let url = Url::parse("https://example.com/one/two.html").unwrap();

    // root relative url
    let html_bytes = b"<!DOCTYPE html><html><head><meta property='og:image' content='/image.jpg'></head><body></body></html>";
    let metadata = html_to_site_metadata(html_bytes, &url).expect("Unable to parse metadata");
    assert_eq!(
      metadata.image,
      Some(Url::parse("https://example.com/image.jpg").unwrap().into())
    );

    // base relative url
    let html_bytes = b"<!DOCTYPE html><html><head><meta property='og:image' content='image.jpg'></head><body></body></html>";
    let metadata = html_to_site_metadata(html_bytes, &url).expect("Unable to parse metadata");
    assert_eq!(
      metadata.image,
      Some(
        Url::parse("https://example.com/one/image.jpg")
          .unwrap()
          .into()
      )
    );

    // absolute url
    let html_bytes = b"<!DOCTYPE html><html><head><meta property='og:image' content='https://cdn.host.com/image.jpg'></head><body></body></html>";
    let metadata = html_to_site_metadata(html_bytes, &url).expect("Unable to parse metadata");
    assert_eq!(
      metadata.image,
      Some(Url::parse("https://cdn.host.com/image.jpg").unwrap().into())
    );

    // protocol relative url
    let html_bytes = b"<!DOCTYPE html><html><head><meta property='og:image' content='//example.com/image.jpg'></head><body></body></html>";
    let metadata = html_to_site_metadata(html_bytes, &url).expect("Unable to parse metadata");
    assert_eq!(
      metadata.image,
      Some(Url::parse("https://example.com/image.jpg").unwrap().into())
    );
  }
}
