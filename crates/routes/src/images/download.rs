use super::utils::{adapt_request, convert_header};
use actix_web::{
  HttpRequest,
  HttpResponse,
  HttpResponseBuilder,
  Responder,
  body::{BodyStream, BoxBody},
  http::{StatusCode, header::CONTENT_DISPOSITION},
  web::{Data, *},
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::{images::RemoteImage, local_site::LocalSite};
use lemmy_db_views_local_image::api::{ImageGetParams, ImageProxyParams};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use std::str::FromStr;
use strum::{Display, EnumString};
use url::Url;

pub async fn get_image(
  filename: Path<String>,
  Query(params): Query<ImageGetParams>,
  req: HttpRequest,
  local_user_view: Option<LocalUserView>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  if is_auth_required(local_user_view.as_ref(), &local_site) {
    return Ok(HttpResponse::Unauthorized().finish());
  }

  let name = filename.into_inner();

  // If there are no query params, the URL is original
  let pictrs_url = context.settings().pictrs()?.url;
  let processed_url = if params.file_type.is_none() && params.max_size.is_none() {
    format!("{}image/original/{}", pictrs_url, name)
  } else {
    let file_type = file_type(params.file_type, &name).unwrap_or_default();

    let mut url = format!("{}image/process.{}?src={}", pictrs_url, file_type, name);

    if let Some(size) = params.max_size {
      url = format!("{url}&thumbnail={size}",);
    }
    url
  };

  do_get_image(processed_url, req, &context, Some(name)).await
}

pub async fn image_proxy(
  Query(params): Query<ImageProxyParams>,
  req: HttpRequest,
  local_user_view: Option<LocalUserView>,
  context: Data<LemmyContext>,
) -> LemmyResult<Either<HttpResponse<()>, HttpResponse<BoxBody>>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  if is_auth_required(local_user_view.as_ref(), &local_site) {
    return Ok(Either::Right(HttpResponse::Unauthorized().finish()));
  }

  let url = Url::parse(&params.url)?;
  let encoded_url = utf8_percent_encode(&params.url, NON_ALPHANUMERIC).to_string();

  // Check that url corresponds to a federated image so that this can't be abused as a proxy
  // for arbitrary purposes.
  RemoteImage::validate(&mut context.pool(), url.clone().into()).await?;

  // If the URL is itself a proxy URL from another Lemmy instance (which happens when
  // a federated post's thumbnail_url is re-proxied), unwrap it to get the original image
  // URL for correct filename and file-type derivation.
  let url_for_filename = unwrap_proxy_url(&url);

  let pictrs_config = context.settings().pictrs()?;
  let output_file_type = (params.file_type.is_some() || params.max_size.is_some())
    .then(|| file_type(params.file_type.clone(), url_for_filename.path()).unwrap_or_default());

  let processed_url = if let Some(file_type) = &output_file_type {
    let mut url = format!(
      "{}image/process.{}?proxy={}",
      pictrs_config.url, file_type, encoded_url
    );

    if let Some(size) = params.max_size {
      url = format!("{url}&thumbnail={size}",);
    }
    url
  } else {
    format!("{}image/original?proxy={}", pictrs_config.url, encoded_url)
  };

  let proxy_bypass_domains = SiteView::read_local(&mut context.pool())
    .await?
    .local_site
    .image_proxy_bypass_domains
    .map(|e| e.split(',').map(ToString::to_string).collect::<Vec<_>>())
    .unwrap_or_default();

  let bypass_proxy = proxy_bypass_domains
    .iter()
    .any(|s| url.domain().is_some_and(|d| d == s));

  if bypass_proxy {
    // Bypass proxy and redirect user to original image
    Ok(Either::Left(Redirect::to(url.to_string()).respond_to(&req)))
  } else {
    // Proxy the image data through Lemmy
    let download_filename =
      download_filename_from_url_path(url_for_filename.path(), output_file_type);
    Ok(Either::Right(
      do_get_image(processed_url, req, &context, download_filename).await?,
    ))
  }
}

pub(super) async fn do_get_image(
  url: String,
  req: HttpRequest,
  context: &LemmyContext,
  download_filename: Option<String>,
) -> LemmyResult<HttpResponse> {
  let mut client_req = adapt_request(&req, url, context);

  if let Some(addr) = req.head().peer_addr {
    client_req = client_req.header("X-Forwarded-For", addr.to_string());
  }

  let res = client_req.send().await?;

  if res.status() == http::StatusCode::NOT_FOUND {
    return Ok(HttpResponse::NotFound().finish());
  }

  let mut client_res = HttpResponse::build(StatusCode::from_u16(res.status().as_u16())?);

  for (name, value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
    client_res.insert_header(convert_header(name, value));
  }

  if let Some(download_filename) = &download_filename {
    set_content_disposition(&mut client_res, download_filename);
  }

  let built_response = client_res.body(BodyStream::new(res.bytes_stream()));
  Ok(built_response)
}

/// Auth required if instance is private with federation disabled
fn is_auth_required(local_user_view: Option<&LocalUserView>, local_site: &LocalSite) -> bool {
  local_user_view.is_none() && local_site.private_instance && !local_site.federation_enabled
}

#[derive(EnumString, Display, PartialEq, Debug, Default)]
#[strum(ascii_case_insensitive, serialize_all = "snake_case")]
enum PictrsFileType {
  Apng,
  Avif,
  Gif,
  #[default]
  Jpg,
  Jxl,
  Png,
  Webp,
}

fn set_content_disposition(client_res: &mut HttpResponseBuilder, filename: &str) {
  let encoded = urlencoding::encode(filename);
  let header_value = format!("inline; filename=\"{}\"", encoded);
  client_res.insert_header((CONTENT_DISPOSITION, header_value));
}

/// If the given URL is itself a Lemmy image proxy URL, recursively extract the
/// original image URL from its `url` query param. This handles federated posts
/// where the thumbnail URL from the origin instance is already a proxy URL that
/// gets re-proxied by the receiving instance.
fn is_proxy_url(path: &str) -> bool {
  path.ends_with("/api/v4/image/proxy") || path.ends_with("/api/v3/image_proxy")
}

fn unwrap_proxy_url(url: &Url) -> Url {
  if !is_proxy_url(url.path()) {
    return url.clone();
  }

  let inner = url
    .query_pairs()
    .find(|(k, _)| k == "url")
    .map(|(_, v)| v)
    .and_then(|v| Url::parse(&v).ok());

  match inner {
    Some(inner) => unwrap_proxy_url(&inner),
    None => url.clone(),
  }
}

/// Extracts the final path segment from a URL, percent-decodes it, and returns a
/// download filename.
///
/// If `output_file_type` is set, the extension is replaced with that type.
/// Otherwise the original extension is preserved, or `.jpg` is added when none exists.
fn download_filename_from_url_path(
  path: &str,
  output_file_type: Option<PictrsFileType>,
) -> Option<String> {
  let raw = path
    .rsplit('/')
    .next()?
    .split('?')
    .next()
    .filter(|s| !s.is_empty())?;
  let decoded = urlencoding::decode(raw).unwrap_or_else(|_| raw.into());
  let name = decoded.as_ref();

  let has_ext = name.rsplit_once('.').is_some_and(|(s, _)| !s.is_empty());
  let stem = name
    .rsplit_once('.')
    .filter(|(s, _)| !s.is_empty())
    .map_or(name, |(s, _)| s);
  match output_file_type {
    None if has_ext => Some(name.into()),
    None => Some(format!("{name}.jpg")),
    Some(ft) => Some(format!("{stem}.{ft}")),
  }
}

/// Take file type from param, name, or use jpg if nothing is given
fn file_type(file_type: Option<String>, name: &str) -> LemmyResult<PictrsFileType> {
  let type_str = file_type
    .clone()
    .unwrap_or_else(|| name.split('.').next_back().unwrap_or("jpg").to_string());

  PictrsFileType::from_str(&type_str).with_lemmy_type(LemmyErrorType::NotAnImageType)
}

#[cfg(test)]
mod tests {
  use super::{PictrsFileType, download_filename_from_url_path, set_content_disposition};
  use crate::images::download::file_type;
  use actix_web::{
    HttpResponse,
    http::{StatusCode, header::CONTENT_DISPOSITION},
  };
  use lemmy_utils::error::LemmyResult;

  #[tokio::test]
  async fn image_file_type_tests() -> LemmyResult<()> {
    // Make sure files type outputs are getting lower-cased
    assert_eq!(PictrsFileType::Jpg.to_string(), "jpg".to_string());

    let file_url = "a8a7f07f-3ef2-40fa-849c-ae952f68f3ec.jpg";

    // Make sure wrong-cased file type requests are okay
    assert_eq!(
      PictrsFileType::Jpg,
      file_type(Some("JPg".to_string()), file_url)?
    );

    // Make sure converts are working
    assert_eq!(
      PictrsFileType::Avif,
      file_type(Some("AVif".to_string()), file_url)?
    );

    // Make sure wrong file type requests are okay with unwrap_or_default
    assert_eq!(
      PictrsFileType::Jpg,
      file_type(Some("jpeg".to_string()), file_url).unwrap_or_default()
    );
    assert_eq!(
      PictrsFileType::Jpg,
      file_type(Some("nonsense".to_string()), file_url).unwrap_or_default()
    );

    // Make sure missing file type requests are okay
    assert_eq!(PictrsFileType::Jpg, file_type(None, file_url)?);

    // jpeg
    let file_url = "a8a7f07f-3ef2-40fa-849c-ae952f68f3ec.jpeg";

    // Make sure jpeg one is okay
    assert_eq!(
      PictrsFileType::Jpg,
      file_type(None, file_url).unwrap_or_default()
    );

    // Make sure proxy ones are okay
    let proxy_url = "https://test.tld/pictrs/image/6d3b2f3f-7b29-4d9a-868e-b269423f4d6c.WEbP";
    assert_eq!(PictrsFileType::Webp, file_type(None, proxy_url)?);

    Ok(())
  }

  #[test]
  fn test_download_filename_from_url_path() {
    assert_eq!(
      download_filename_from_url_path("/images/photo.png", Some(PictrsFileType::Avif)),
      Some("photo.avif".to_string())
    );

    assert_eq!(
      download_filename_from_url_path("/images/archive", Some(PictrsFileType::Webp)),
      Some("archive.webp".to_string())
    );

    assert_eq!(
      download_filename_from_url_path("/images/photo.tar.gz", Some(PictrsFileType::Jpg)),
      Some("photo.tar.jpg".to_string())
    );

    assert_eq!(
      download_filename_from_url_path("/images/%C3%A9l%C3%A9phant.png", Some(PictrsFileType::Jxl)),
      Some("éléphant.jxl".to_string())
    );

    // Without output file type, original extension is preserved
    assert_eq!(
      download_filename_from_url_path("/images/photo.png", None),
      Some("photo.png".to_string())
    );

    // Without output file type and no extension, falls back to .jpg
    assert_eq!(
      download_filename_from_url_path("/images/noextension", None),
      Some("noextension.jpg".to_string())
    );
  }

  #[test]
  fn test_set_content_disposition() {
    let mut builder = HttpResponse::build(StatusCode::OK);

    // ASCII filename: URL-encoded, preserving characters allowed by urlencoding::encode
    set_content_disposition(&mut builder, "photo.jpg");
    let res = builder.finish();
    assert_eq!(
      res
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|header| header.to_str().ok()),
      Some("inline; filename=\"photo.jpg\"")
    );

    // Spaces are encoded
    let mut builder2 = HttpResponse::build(StatusCode::OK);
    set_content_disposition(&mut builder2, "my photo.jpg");
    let res2 = builder2.finish();
    assert_eq!(
      res2
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|header| header.to_str().ok()),
      Some("inline; filename=\"my%20photo.jpg\"")
    );

    // Non-ASCII characters are UTF-8 percent-encoded
    let mut builder3 = HttpResponse::build(StatusCode::OK);
    set_content_disposition(&mut builder3, "héron.jpg");
    let res3 = builder3.finish();
    assert_eq!(
      res3
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|header| header.to_str().ok()),
      Some("inline; filename=\"h%C3%A9ron.jpg\"")
    );
  }
}
