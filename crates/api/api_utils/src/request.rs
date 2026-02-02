use crate::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::proxy_image_link,
};
use activitypub_federation::config::Data;
use chrono::{DateTime, Utc};
use encoding_rs::{Encoding, UTF_8};
use futures::StreamExt;
use lemmy_db_schema::source::{
  images::{ImageDetailsInsertForm, LocalImage, LocalImageForm},
  post::{Post, PostUpdateForm},
  site::Site,
};
use lemmy_db_views_post::api::{LinkMetadata, OpenGraphData};
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::{
  REQWEST_TIMEOUT,
  VERSION,
  error::{LemmyError, LemmyErrorExt, LemmyErrorType, LemmyResult, UntranslatedError},
  settings::structs::{PictrsImageMode, Settings},
};
use mime::{Mime, TEXT_HTML};
use reqwest::{
  Client,
  ClientBuilder,
  Response,
  header::{CONTENT_TYPE, LOCATION, RANGE},
  redirect::Policy,
};
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use tokio::net::lookup_host;
use tracing::{info, warn};
use url::Url;
use urlencoding::encode;
use webpage::{HTML, OpengraphObject};

pub fn client_builder(settings: &Settings) -> ClientBuilder {
  // https://github.com/seanmonstar/reqwest/issues/2924
  let _ = rustls::crypto::ring::default_provider().install_default();

  let user_agent = format!(
    "Lemmy/{}; +{}",
    *VERSION,
    settings.get_protocol_and_hostname()
  );

  Client::builder()
    .user_agent(user_agent.clone())
    .timeout(REQWEST_TIMEOUT)
    .connect_timeout(REQWEST_TIMEOUT)
    .redirect(Policy::none())
}

/// Fetches metadata for the given link and optionally generates thumbnail.
pub async fn fetch_link_metadata(
  url: &Url,
  context: &LemmyContext,
  recursion: bool,
) -> LemmyResult<LinkMetadata> {
  if url.scheme() != "http" && url.scheme() != "https" {
    return Err(LemmyErrorType::InvalidUrl.into());
  }

  // Resolve the domain and throw an error if it points to any internal IP,
  // using logic from nightly IpAddr::is_global.
  if !cfg!(debug_assertions) {
    // TODO: Replace with IpAddr::is_global() once stabilized
    //       https://doc.rust-lang.org/std/net/enum.IpAddr.html#method.is_global
    let domain = url.domain().ok_or(UntranslatedError::UrlWithoutDomain)?;
    let invalid_ip = lookup_host((domain.to_owned(), 80))
      .await?
      .any(|addr| match addr.ip() {
        IpAddr::V4(addr) => {
          addr.is_private() || addr.is_link_local() || addr.is_loopback() || addr.is_multicast()
        }
        IpAddr::V6(addr) => {
          addr.is_loopback()
                        || addr.is_multicast()
                        || ((addr.segments()[0] & 0xfe00) == 0xfc00) // is_unique_local
                        || ((addr.segments()[0] & 0xffc0) == 0xfe80) // is_unicast_link_local
        }
      });
    if invalid_ip {
      return Err(LemmyErrorType::InvalidUrl.into());
    }
  }

  info!("Fetching site metadata for url: {}", url);
  // We only fetch the first MB of data in order to not waste bandwidth especially for large
  // binary files. This high limit is particularly needed for youtube, which includes a lot of
  // javascript code before the opengraph tags. Mastodon also uses a 1 MB limit:
  // https://github.com/mastodon/mastodon/blob/295ad6f19a016b3f16e1201ffcbb1b3ad6b455a2/app/lib/request.rb#L213
  let bytes_to_fetch = 1024 * 1024;
  let response = context
    .client()
    .get(url.as_str())
    // we only need the first chunk of data. Note that we do not check for Accept-Range so the
    // server may ignore this and still respond with the full response
    .header(RANGE, format!("bytes=0-{}", bytes_to_fetch - 1)) /* -1 because inclusive */
    .send()
    .await?
    .error_for_status()?;

  // Manually follow one redirect, using internal IP check. Further redirects are ignored.
  let location = response
    .headers()
    .get(LOCATION)
    .and_then(|l| l.to_str().ok());
  if let (Some(location), false) = (location, recursion) {
    let url = location.parse()?;
    return Box::pin(fetch_link_metadata(&url, context, true)).await;
  }

  let mut content_type: Option<Mime> = response
    .headers()
    .get(CONTENT_TYPE)
    .and_then(|h| h.to_str().ok())
    .and_then(|h| h.parse().ok())
    // If we don't get a content_type from the response (e.g. if the server is down),
    // then try to infer the content_type from the file extension.
    .or(mime_guess::from_path(url.path()).first());

  let opengraph_data = {
    let is_html = content_type
      .as_ref()
      .map(|c| {
        // application/xhtml+xml is a subset of HTML
        let application_xhtml: Mime = "application/xhtml+xml".parse::<Mime>().unwrap_or(TEXT_HTML);
        let allowed_mime_types = [TEXT_HTML.essence_str(), application_xhtml.essence_str()];
        allowed_mime_types.contains(&c.essence_str())
      })
      .unwrap_or_default();

    if is_html {
      // Can't use .text() here, because it only checks the content header, not the actual bytes
      // https://github.com/LemmyNet/lemmy/issues/1964
      // So we want to do deep inspection of the actually returned bytes but need to be careful
      // not spend too much time parsing binary data as HTML
      // only take first bytes regardless of how many bytes the server returns
      let html_bytes = collect_bytes_until_limit(response, bytes_to_fetch).await?;
      extract_opengraph_data(&html_bytes, url)
        .map_err(|e| info!("{e}"))
        .unwrap_or_default()
    } else {
      let is_octet_type = content_type
        .as_ref()
        .map(|c| c.subtype() == "octet-stream")
        .unwrap_or_default();

      // Overwrite the content type if its an octet type
      if is_octet_type {
        // Don't need to fetch as much data for this as we do with opengraph
        let octet_bytes = collect_bytes_until_limit(response, 512).await?;
        content_type =
          infer::get(&octet_bytes).map_or(content_type, |t| t.mime_type().parse().ok());
      }

      Default::default()
    }
  };

  Ok(LinkMetadata {
    opengraph_data,
    content_type: content_type.map(|c| c.to_string()),
  })
}

async fn collect_bytes_until_limit(
  response: Response,
  requested_bytes: usize,
) -> Result<Vec<u8>, LemmyError> {
  let mut stream = response.bytes_stream();
  let mut bytes = Vec::with_capacity(requested_bytes);
  while let Some(chunk) = stream.next().await {
    let chunk = chunk.map_err(LemmyError::from)?;
    // we may go over the requested size here but the important part is we don't keep aggregating
    // more chunks than needed
    bytes.extend_from_slice(&chunk);
    if bytes.len() >= requested_bytes {
      bytes.truncate(requested_bytes);
      break;
    }
  }
  Ok(bytes)
}

/// Generates and saves a post thumbnail and metadata.
///
/// Takes a callback to generate a send activity task, so that post can be federated with metadata.
///
/// TODO: `federated_thumbnail` param can be removed once we federate full metadata and can
///       write it to db directly, without calling this function.
///       https://github.com/LemmyNet/lemmy/issues/4598
pub async fn generate_post_link_metadata(
  post: Post,
  custom_thumbnail: Option<Url>,
  send_activity: impl FnOnce(Post) -> Option<SendActivityData> + Send + 'static,
  context: Data<LemmyContext>,
) -> LemmyResult<()> {
  let metadata = match &post.url {
    Some(url) => fetch_link_metadata(url, &context, false)
      .await
      .unwrap_or_default(),
    _ => Default::default(),
  };

  let is_image_post = metadata
    .content_type
    .as_ref()
    .is_some_and(|content_type| content_type.starts_with("image"));

  // Decide if we are allowed to generate local thumbnail
  let site = Site::read_local(&mut context.pool()).await?;
  let allow_sensitive = site.content_warning.is_some();
  let allow_generate_thumbnail = allow_sensitive || !post.nsfw;

  // Proxy the post url itself if it is an image
  let url = if let (true, Some(url)) = (is_image_post, post.url.clone()) {
    Some(Some(proxy_image_link(url.into(), false, &context).await?))
  } else {
    None
  };

  let image_url = if is_image_post {
    post.url.clone()
  } else {
    metadata.opengraph_data.image.clone()
  };

  // Attempt to generate a thumbnail depending on the instance settings. Either by proxying,
  // storing image persistently in pict-rs or returning the remote url directly as thumbnail.
  let thumbnail_url = if let (false, Some(url)) = (is_image_post, custom_thumbnail) {
    proxy_image_link(url.clone(), true, &context)
      .await
      .map_err(|e| warn!("Failed to proxy thumbnail: {e}"))
      .ok()
      .or(Some(url.into()))
  } else if let (true, Some(url)) = (allow_generate_thumbnail, image_url.clone()) {
    generate_pictrs_thumbnail(&post, &url, &context)
      .await
      .map_err(|e| warn!("Failed to generate thumbnail: {e}"))
      .ok()
      .map(Into::into)
      .or(image_url)
  } else {
    image_url.clone()
  };

  let form = PostUpdateForm {
    url,
    embed_title: Some(metadata.opengraph_data.title),
    embed_description: Some(metadata.opengraph_data.description),
    embed_video_url: Some(metadata.opengraph_data.embed_video_url),
    embed_video_width: Some(metadata.opengraph_data.video_width.map(i32::from)),
    embed_video_height: Some(metadata.opengraph_data.video_height.map(i32::from)),
    thumbnail_url: Some(thumbnail_url),
    url_content_type: Some(metadata.content_type),
    ..Default::default()
  };
  let updated_post = Post::update(&mut context.pool(), post.id, &form).await?;
  if let Some(send_activity) = send_activity(updated_post) {
    ActivityChannel::submit_activity(send_activity, &context)?;
  }
  Ok(())
}

/// Extract site metadata from HTML Opengraph attributes.
fn extract_opengraph_data(html_bytes: &[u8], url: &Url) -> LemmyResult<OpenGraphData> {
  let html = String::from_utf8_lossy(html_bytes);

  let mut page = HTML::from_string(html.to_string(), None)?;

  // If the web page specifies that it isn't actually UTF-8, re-decode the received bytes with the
  // proper encoding. If the specified encoding cannot be found, fall back to the original UTF-8
  // version.
  if let Some(charset) = page.meta.get("charset")
    && charset != UTF_8.name()
    && let Some(encoding) = Encoding::for_label(charset.as_bytes())
  {
    page = HTML::from_string(encoding.decode(html_bytes).0.into(), None)?;
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
    .filter(|v| !v.url.is_empty())
    // join also works if the target URL is absolute
    .and_then(|ogo| url.join(&ogo.url).ok());

  let (og_image_width, og_image_height) =
    extract_opengraph_width_and_height(page.opengraph.images.first());

  let og_embed_url = page
    .opengraph
    .videos
    .first()
    // Sometime sites provide `og:video` tags with empty content
    .filter(|v| !v.url.is_empty())
    // join also works if the target URL is absolute
    .and_then(|v| url.join(&v.url).ok());

  let (og_video_width, og_video_height) =
    extract_opengraph_width_and_height(page.opengraph.videos.first());

  Ok(OpenGraphData {
    title: og_title.or(page_title),
    description: og_description.or(page_description),
    image: og_image.map(Into::into),
    image_width: og_image_width,
    image_height: og_image_height,
    embed_video_url: og_embed_url.map(Into::into),
    video_width: og_video_width,
    video_height: og_video_height,
  })
}

fn extract_opengraph_width_and_height(ogo: Option<&OpengraphObject>) -> (Option<u16>, Option<u16>) {
  (
    ogo.and_then(|ogo| extract_opengraph_int_field(ogo, "width")),
    ogo.and_then(|ogo| extract_opengraph_int_field(ogo, "height")),
  )
}

fn extract_opengraph_int_field(ogo: &OpengraphObject, field: &str) -> Option<u16> {
  ogo
    .properties
    .get(field)
    .and_then(|w| w.parse::<u16>().ok())
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PictrsResponse {
  #[serde(default)]
  pub files: Vec<PictrsFile>,
  pub msg: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PictrsFile {
  pub file: String,
  pub details: PictrsFileDetails,
}

impl PictrsFile {
  pub fn image_url(&self, protocol_and_hostname: &str) -> Result<Url, url::ParseError> {
    Url::parse(&format!(
      "{protocol_and_hostname}/api/v4/image/{}",
      self.file
    ))
  }
}

/// Stores extra details about a Pictrs image.
#[derive(Deserialize, Serialize, Debug)]
pub struct PictrsFileDetails {
  /// In pixels
  pub width: u16,
  /// In pixels
  pub height: u16,
  pub content_type: String,
  pub created_at: DateTime<Utc>,
  pub blurhash: Option<String>,
}

impl PictrsFileDetails {
  /// Builds the image form. This should always use the thumbnail_url,
  /// Because the post_view joins to it
  pub fn build_image_details_form(&self, thumbnail_url: &Url) -> ImageDetailsInsertForm {
    ImageDetailsInsertForm {
      link: thumbnail_url.clone().into(),
      width: self.width.into(),
      height: self.height.into(),
      content_type: self.content_type.clone(),
      blurhash: self.blurhash.clone(),
    }
  }
}

#[derive(Deserialize, Serialize, Debug)]
struct PictrsPurgeResponse {
  msg: String,
  aliases: Vec<String>,
}

/// Purges an image from pictrs
/// Note: This should often be coerced from a Result to .ok() in order to fail softly, because:
/// - It might fail due to image being not local
/// - It might not be an image
/// - Pictrs might not be set up
pub async fn purge_image_from_pictrs_url(
  image_url: &Url,
  context: &LemmyContext,
) -> LemmyResult<()> {
  is_image_content_type(context.pictrs_client(), image_url).await?;

  let alias = image_url
    .path_segments()
    .ok_or(UntranslatedError::PurgeInvalidImageUrl)?
    .next_back()
    .ok_or(UntranslatedError::PurgeInvalidImageUrl)?;

  purge_image_from_pictrs(alias, context).await
}

pub async fn purge_image_from_pictrs(alias: &str, context: &LemmyContext) -> LemmyResult<()> {
  let pictrs_config = context.settings().pictrs()?;
  let purge_url = format!("{}internal/purge?alias={}", pictrs_config.url, alias);

  let pictrs_api_key = pictrs_config
    .api_key
    .ok_or(LemmyErrorType::PictrsApiKeyNotProvided)?;
  let response = context
    .pictrs_client()
    .post(&purge_url)
    .timeout(REQWEST_TIMEOUT)
    .header("x-api-token", pictrs_api_key)
    .send()
    .await?
    .error_for_status()?;

  let response: PictrsPurgeResponse = response.json().await.map_err(LemmyError::from)?;

  // Pictrs purges return all aliases.
  let aliases = response.aliases;

  // Delete db rows of aliases.
  LocalImage::delete_by_aliases(&mut context.pool(), &aliases)
    .await
    .ok();

  match response.msg.as_str() {
    "ok" => Ok(()),
    _ => Err(LemmyErrorType::PictrsPurgeResponseError(response.msg))?,
  }
}

/// Deletes an alias for an image from the local db and pictrs. If it's not the last / only alias,
/// the image might remain.
///
/// # Security Warning
/// This is a low-level function that doesn't check if the user is allowed to delete the image
/// alias. Callers MUST check if the user has permission to delete the alias
/// before calling this function (the user is an admin or the image belongs to the user).
pub async fn delete_image_alias(alias: &str, context: &LemmyContext) -> LemmyResult<()> {
  let pictrs_config = context.settings().pictrs()?;
  let url = format!("{}internal/delete?alias={}", pictrs_config.url, &alias);

  // Send the delete request to pictrs.
  context
    .pictrs_client()
    .post(&url)
    .header("X-Api-Token", pictrs_config.api_key.unwrap_or_default())
    .timeout(REQWEST_TIMEOUT)
    .send()
    .await?
    .error_for_status()?;

  // Delete db row if any (old Lemmy versions didn't generate this).
  LocalImage::delete_by_alias(&mut context.pool(), alias)
    .await
    .ok();
  Ok(())
}

/// Retrieves the image with local pict-rs and generates a thumbnail. Returns the thumbnail url.
async fn generate_pictrs_thumbnail(
  post: &Post,
  image_url: &Url,
  context: &LemmyContext,
) -> LemmyResult<Url> {
  let pictrs_config = context.settings().pictrs()?;

  match pictrs_config.image_mode {
    PictrsImageMode::None => return Ok(image_url.clone()),
    PictrsImageMode::ProxyAllImages => {
      return Ok(
        proxy_image_link(image_url.clone(), true, context)
          .await?
          .into(),
      );
    }
    _ => {}
  };

  // fetch remote non-pictrs images for persistent thumbnail link
  let fetch_url = format!(
    "{}image/download?url={}&resize={}",
    pictrs_config.url,
    encode(image_url.as_str()),
    context.settings().pictrs()?.max_thumbnail_size
  );

  let res = context
    .pictrs_client()
    .get(&fetch_url)
    .timeout(REQWEST_TIMEOUT)
    .send()
    .await?
    .error_for_status()?
    .json::<PictrsResponse>()
    .await?;

  let image = res
    .files
    .first()
    .ok_or(LemmyErrorType::PictrsResponseError(res.msg))?;

  let form = LocalImageForm {
    pictrs_alias: image.file.clone(),
    // For thumbnails, the person_id is the post creator
    person_id: post.creator_id,
    thumbnail_for_post_id: Some(Some(post.id)),
  };
  let protocol_and_hostname = context.settings().get_protocol_and_hostname();
  let thumbnail_url = image.image_url(&protocol_and_hostname)?;

  // Also store the details for the image
  let details_form = image.details.build_image_details_form(&thumbnail_url);
  LocalImage::create(&mut context.pool(), &form, &details_form).await?;

  Ok(thumbnail_url)
}

/// Fetches the image details for pictrs proxied images
///
/// We don't need to check for image mode, as that's already been done
pub async fn fetch_pictrs_proxied_image_details(
  image_url: &Url,
  context: &LemmyContext,
) -> LemmyResult<PictrsFileDetails> {
  let pictrs_url = context.settings().pictrs()?.url;
  let encoded_image_url = encode(image_url.as_str());

  // Pictrs needs you to fetch the proxied image before you can fetch the details
  let proxy_url = format!("{pictrs_url}image/original?proxy={encoded_image_url}");

  context
    .pictrs_client()
    .get(&proxy_url)
    .timeout(REQWEST_TIMEOUT)
    .send()
    .await?
    .error_for_status()
    .with_lemmy_type(LemmyErrorType::NotAnImageType)?;

  let details_url = format!("{pictrs_url}image/details/original?proxy={encoded_image_url}");

  let res = context
    .pictrs_client()
    .get(&details_url)
    .timeout(REQWEST_TIMEOUT)
    .send()
    .await?
    .error_for_status()?
    .json()
    .await?;

  Ok(res)
}

// TODO: get rid of this by reading content type from db

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

#[cfg(test)]
mod tests {

  use crate::{
    context::LemmyContext,
    request::{extract_opengraph_data, fetch_link_metadata},
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;
  use url::Url;

  // These helped with testing
  #[tokio::test]
  #[serial]
  async fn test_link_metadata() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let sample_url = Url::parse("https://gitlab.com/IzzyOnDroid/repo/-/wikis/FAQ")?;
    let sample_res = fetch_link_metadata(&sample_url, &context, false).await?;
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
        Url::parse("https://gitlab.com/uploads/-/system/project/avatar/4877469/iod_logo.png")?
          .into()
      ),
      sample_res.opengraph_data.image
    );
    assert_eq!(None, sample_res.opengraph_data.embed_video_url);
    assert_eq!(
      Some(mime::TEXT_HTML_UTF_8.to_string()),
      sample_res.content_type
    );

    Ok(())
  }

  #[test]
  fn test_resolve_image_url() -> LemmyResult<()> {
    // url that lists the opengraph fields
    let url = Url::parse("https://example.com/one/two.html")?;

    // root relative url
    let html_bytes = b"<!DOCTYPE html><html><head><meta property='og:image' content='/image.jpg'></head><body></body></html>";
    let metadata = extract_opengraph_data(html_bytes, &url)?;
    assert_eq!(
      metadata.image,
      Some(Url::parse("https://example.com/image.jpg")?.into())
    );

    // base relative url
    let html_bytes = b"<!DOCTYPE html><html><head><meta property='og:image' content='image.jpg'></head><body></body></html>";
    let metadata = extract_opengraph_data(html_bytes, &url)?;
    assert_eq!(
      metadata.image,
      Some(Url::parse("https://example.com/one/image.jpg")?.into())
    );

    // absolute url
    let html_bytes = b"<!DOCTYPE html><html><head><meta property='og:image' content='https://cdn.host.com/image.jpg'></head><body></body></html>";
    let metadata = extract_opengraph_data(html_bytes, &url)?;
    assert_eq!(
      metadata.image,
      Some(Url::parse("https://cdn.host.com/image.jpg")?.into())
    );

    // protocol relative url
    let html_bytes = b"<!DOCTYPE html><html><head><meta property='og:image' content='//example.com/image.jpg'></head><body></body></html>";
    let metadata = extract_opengraph_data(html_bytes, &url)?;
    assert_eq!(
      metadata.image,
      Some(Url::parse("https://example.com/image.jpg")?.into())
    );

    // image width and height
    let html_bytes = b"<!DOCTYPE html><html><head><meta property='og:image' content='/image.jpg'><meta property='og:image:width' content='400' /><meta property='og:image:height' content='200' /></head><body></body></html>";
    let metadata = extract_opengraph_data(html_bytes, &url)?;
    assert_eq!(
      (metadata.image_width, metadata.image_height),
      (Some(400), Some(200))
    );

    // Empty urls shouldn't return anything
    let html_bytes = b"<!DOCTYPE html><html><head><meta property='og:image' content=''></head><body></body></html>";
    let metadata = extract_opengraph_data(html_bytes, &url)?;
    assert_eq!(metadata.image, None);

    Ok(())
  }
}
