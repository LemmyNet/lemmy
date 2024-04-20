use crate::{
  context::LemmyContext,
  lemmy_db_schema::traits::Crud,
  post::{LinkMetadata, OpenGraphData},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{local_site_opt_to_sensitive, proxy_image_link, proxy_image_link_opt_apub},
};
use activitypub_federation::config::Data;
use encoding::{all::encodings, DecoderTrap};
use lemmy_db_schema::{
  newtypes::DbUrl,
  source::{
    images::{LocalImage, LocalImageForm},
    local_site::LocalSite,
    post::{Post, PostUpdateForm},
  },
};
use lemmy_utils::{
  error::{LemmyError, LemmyErrorType, LemmyResult},
  settings::structs::{PictrsImageMode, Settings},
  spawn_try_task,
  REQWEST_TIMEOUT,
  VERSION,
};
use mime::Mime;
use reqwest::{header::CONTENT_TYPE, Client, ClientBuilder};
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;
use tracing::info;
use url::Url;
use urlencoding::encode;
use webpage::HTML;

pub fn client_builder(settings: &Settings) -> ClientBuilder {
  let user_agent = format!("Lemmy/{VERSION}; +{}", settings.get_protocol_and_hostname());

  Client::builder()
    .user_agent(user_agent.clone())
    .timeout(REQWEST_TIMEOUT)
    .connect_timeout(REQWEST_TIMEOUT)
}

/// Fetches metadata for the given link and optionally generates thumbnail.
#[tracing::instrument(skip_all)]
pub async fn fetch_link_metadata(url: &Url, context: &LemmyContext) -> LemmyResult<LinkMetadata> {
  info!("Fetching site metadata for url: {}", url);
  let response = context.client().get(url.as_str()).send().await?;

  let content_type: Option<Mime> = response
    .headers()
    .get(CONTENT_TYPE)
    .and_then(|h| h.to_str().ok())
    .and_then(|h| h.parse().ok());

  // Can't use .text() here, because it only checks the content header, not the actual bytes
  // https://github.com/LemmyNet/lemmy/issues/1964
  let html_bytes = response.bytes().await.map_err(LemmyError::from)?.to_vec();

  let opengraph_data = extract_opengraph_data(&html_bytes, url)
    .map_err(|e| info!("{e}"))
    .unwrap_or_default();
  Ok(LinkMetadata {
    opengraph_data,
    content_type: content_type.map(|c| c.to_string()),
  })
}

/// Generate post thumbnail in background task, because some sites can be very slow to respond.
///
/// Takes a callback to generate a send activity task, so that post can be federated with metadata.
///
/// TODO: `federated_thumbnail` param can be removed once we federate full metadata and can
///       write it to db directly, without calling this function.
///       https://github.com/LemmyNet/lemmy/issues/4598
pub fn generate_post_link_metadata(
  post: Post,
  custom_thumbnail: Option<Url>,
  federated_thumbnail: Option<Url>,
  send_activity: impl FnOnce(Post) -> Option<SendActivityData> + Send + 'static,
  local_site: Option<LocalSite>,
  context: Data<LemmyContext>,
) {
  spawn_try_task(async move {
    let metadata = match &post.url {
      Some(url) => fetch_link_metadata(url, &context).await.unwrap_or_default(),
      _ => Default::default(),
    };

    let is_image_post = metadata
      .content_type
      .as_ref()
      .is_some_and(|content_type| content_type.starts_with("image"));

    // Decide if we are allowed to generate local thumbnail
    let allow_sensitive = local_site_opt_to_sensitive(&local_site);
    let allow_generate_thumbnail = allow_sensitive || !post.nsfw;

    // Use custom thumbnail if available and its not an image post
    let thumbnail_url = if !is_image_post && custom_thumbnail.is_some() {
      custom_thumbnail
    }
    // Use federated thumbnail if available
    else if federated_thumbnail.is_some() {
      federated_thumbnail
    }
    // Generate local thumbnail if allowed
    else if allow_generate_thumbnail {
      match post.url.or(metadata.opengraph_data.image) {
        Some(url) => generate_pictrs_thumbnail(&url, &context).await.ok(),
        None => None,
      }
    }
    // Otherwise use opengraph preview image directly
    else {
      metadata.opengraph_data.image.map(Into::into)
    };

    // Proxy the image fetch if necessary
    let proxied_thumbnail_url = proxy_image_link_opt_apub(thumbnail_url, &context).await?;

    let form = PostUpdateForm {
      embed_title: Some(metadata.opengraph_data.title),
      embed_description: Some(metadata.opengraph_data.description),
      embed_video_url: Some(metadata.opengraph_data.embed_video_url),
      thumbnail_url: Some(proxied_thumbnail_url),
      url_content_type: Some(metadata.content_type),
      ..Default::default()
    };
    let updated_post = Post::update(&mut context.pool(), post.id, &form).await?;
    if let Some(send_activity) = send_activity(updated_post) {
      ActivityChannel::submit_activity(send_activity, &context).await?;
    }
    Ok(())
  });
}

/// Extract site metadata from HTML Opengraph attributes.
fn extract_opengraph_data(html_bytes: &[u8], url: &Url) -> LemmyResult<OpenGraphData> {
  let html = String::from_utf8_lossy(html_bytes);

  // Make sure the first line is doctype html
  let first_line = html
    .trim_start()
    .lines()
    .next()
    .ok_or(LemmyErrorType::NoLinesInHtml)?
    .to_lowercase();

  if !first_line.starts_with("<!doctype html") {
    Err(LemmyErrorType::SiteMetadataPageIsNotDoctypeHtml)?
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

  Ok(OpenGraphData {
    title: og_title.or(page_title),
    description: og_description.or(page_description),
    image: og_image.map(Into::into),
    embed_video_url: og_embed_url.map(Into::into),
  })
}

#[derive(Deserialize, Debug)]
struct PictrsResponse {
  files: Vec<PictrsFile>,
  msg: String,
}

#[derive(Deserialize, Debug)]
struct PictrsFile {
  file: String,
  delete_token: String,
}

#[derive(Deserialize, Debug)]
struct PictrsPurgeResponse {
  msg: String,
}

/// Purges an image from pictrs
/// Note: This should often be coerced from a Result to .ok() in order to fail softly, because:
/// - It might fail due to image being not local
/// - It might not be an image
/// - Pictrs might not be set up
pub async fn purge_image_from_pictrs(image_url: &Url, context: &LemmyContext) -> LemmyResult<()> {
  is_image_content_type(context.client(), image_url).await?;

  let alias = image_url
    .path_segments()
    .ok_or(LemmyErrorType::ImageUrlMissingPathSegments)?
    .next_back()
    .ok_or(LemmyErrorType::ImageUrlMissingLastPathSegment)?;

  let pictrs_config = context.settings().pictrs_config()?;
  let purge_url = format!("{}internal/purge?alias={}", pictrs_config.url, alias);

  let pictrs_api_key = pictrs_config
    .api_key
    .ok_or(LemmyErrorType::PictrsApiKeyNotProvided)?;
  let response = context
    .client()
    .post(&purge_url)
    .timeout(REQWEST_TIMEOUT)
    .header("x-api-token", pictrs_api_key)
    .send()
    .await?;

  let response: PictrsPurgeResponse = response.json().await.map_err(LemmyError::from)?;

  match response.msg.as_str() {
    "ok" => Ok(()),
    _ => Err(LemmyErrorType::PictrsPurgeResponseError(response.msg))?,
  }
}

pub async fn delete_image_from_pictrs(
  alias: &str,
  delete_token: &str,
  context: &LemmyContext,
) -> LemmyResult<()> {
  let pictrs_config = context.settings().pictrs_config()?;
  let url = format!(
    "{}image/delete/{}/{}",
    pictrs_config.url, &delete_token, &alias
  );
  context
    .client()
    .delete(&url)
    .timeout(REQWEST_TIMEOUT)
    .send()
    .await
    .map_err(LemmyError::from)?;
  Ok(())
}

/// Retrieves the image with local pict-rs and generates a thumbnail. Returns the thumbnail url.
#[tracing::instrument(skip_all)]
async fn generate_pictrs_thumbnail(image_url: &Url, context: &LemmyContext) -> LemmyResult<Url> {
  let pictrs_config = context.settings().pictrs_config()?;

  match pictrs_config.image_mode() {
    PictrsImageMode::None => return Ok(image_url.clone()),
    PictrsImageMode::ProxyAllImages => {
      return Ok(proxy_image_link(image_url.clone(), context).await?.into())
    }
    _ => {}
  };

  // fetch remote non-pictrs images for persistent thumbnail link
  // TODO: should limit size once supported by pictrs
  let fetch_url = format!(
    "{}image/download?url={}",
    pictrs_config.url,
    encode(image_url.as_str())
  );

  let response = context
    .client()
    .get(&fetch_url)
    .timeout(REQWEST_TIMEOUT)
    .send()
    .await?;

  let response: PictrsResponse = response.json().await?;

  if response.msg == "ok" {
    let thumbnail_url = Url::parse(&format!(
      "{}/pictrs/image/{}",
      context.settings().get_protocol_and_hostname(),
      response.files.first().expect("missing pictrs file").file
    ))?;
    for uploaded_image in response.files {
      let form = LocalImageForm {
        local_user_id: None,
        pictrs_alias: uploaded_image.file.to_string(),
        pictrs_delete_token: uploaded_image.delete_token.to_string(),
      };
      LocalImage::create(&mut context.pool(), &form).await?;
    }
    Ok(thumbnail_url)
  } else {
    Err(LemmyErrorType::PictrsResponseError(response.msg))?
  }
}

// TODO: get rid of this by reading content type from db
#[tracing::instrument(skip_all)]
async fn is_image_content_type(client: &ClientWithMiddleware, url: &Url) -> LemmyResult<()> {
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

/// When adding a new avatar or similar image, delete the old one.
pub async fn replace_image(
  new_image: &Option<String>,
  old_image: &Option<DbUrl>,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  if new_image.is_some() {
    // Ignore errors because image may be stored externally.
    if let Some(avatar) = &old_image {
      let image = LocalImage::delete_by_url(&mut context.pool(), avatar)
        .await
        .ok();
      if let Some(image) = image {
        delete_image_from_pictrs(&image.pictrs_alias, &image.pictrs_delete_token, context).await?;
      }
    }
  }
  Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use crate::{
    context::LemmyContext,
    request::{extract_opengraph_data, fetch_link_metadata},
  };
  use pretty_assertions::assert_eq;
  use serial_test::serial;
  use url::Url;

  // These helped with testing
  #[tokio::test]
  #[serial]
  async fn test_link_metadata() {
    let context = LemmyContext::init_test_context().await;
    let sample_url = Url::parse("https://gitlab.com/IzzyOnDroid/repo/-/wikis/FAQ").unwrap();
    let sample_res = fetch_link_metadata(&sample_url, &context).await.unwrap();
    assert_eq!(
      Some("FAQ · Wiki · IzzyOnDroid / repo · GitLab".to_string()),
      sample_res.opengraph_data.title
    );
    assert_eq!(
      Some("The F-Droid compatible repo at https://apt.izzysoft.de/fdroid/".to_string()),
      sample_res.opengraph_data.description
    );
    assert_eq!(
      Some(
        Url::parse("https://gitlab.com/uploads/-/system/project/avatar/4877469/iod_logo.png")
          .unwrap()
          .into()
      ),
      sample_res.opengraph_data.image
    );
    assert_eq!(None, sample_res.opengraph_data.embed_video_url);
    assert_eq!(
      Some(mime::TEXT_HTML_UTF_8.to_string()),
      sample_res.content_type
    );
  }

  #[test]
  fn test_resolve_image_url() {
    // url that lists the opengraph fields
    let url = Url::parse("https://example.com/one/two.html").unwrap();

    // root relative url
    let html_bytes = b"<!DOCTYPE html><html><head><meta property='og:image' content='/image.jpg'></head><body></body></html>";
    let metadata = extract_opengraph_data(html_bytes, &url).expect("Unable to parse metadata");
    assert_eq!(
      metadata.image,
      Some(Url::parse("https://example.com/image.jpg").unwrap().into())
    );

    // base relative url
    let html_bytes = b"<!DOCTYPE html><html><head><meta property='og:image' content='image.jpg'></head><body></body></html>";
    let metadata = extract_opengraph_data(html_bytes, &url).expect("Unable to parse metadata");
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
    let metadata = extract_opengraph_data(html_bytes, &url).expect("Unable to parse metadata");
    assert_eq!(
      metadata.image,
      Some(Url::parse("https://cdn.host.com/image.jpg").unwrap().into())
    );

    // protocol relative url
    let html_bytes = b"<!DOCTYPE html><html><head><meta property='og:image' content='//example.com/image.jpg'></head><body></body></html>";
    let metadata = extract_opengraph_data(html_bytes, &url).expect("Unable to parse metadata");
    assert_eq!(
      metadata.image,
      Some(Url::parse("https://example.com/image.jpg").unwrap().into())
    );
  }
}
