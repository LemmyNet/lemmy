use super::utils::{adapt_request, convert_header};
use actix_web::{
  HttpRequest, HttpResponse, Responder,
  body::{BodyStream, BoxBody},
  http::StatusCode,
  web::{Data, *},
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::images::RemoteImage;
use lemmy_db_views_local_image::api::{ImageGetParams, ImageProxyParams};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use percent_encoding::{NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};
use std::str::FromStr;
use strum::{Display, EnumString};
use url::Url;

pub async fn get_image(
  filename: Path<String>,
  Query(params): Query<ImageGetParams>,
  req: HttpRequest,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let name = &filename.into_inner();

  // If there are no query params, the URL is original
  let pictrs_url = context.settings().pictrs()?.url;
  let processed_url = if params.file_type.is_none() && params.max_size.is_none() {
    format!("{}image/original/{}", pictrs_url, name)
  } else {
    let file_type = file_type(params.file_type, name).unwrap_or_default();

    let mut url = format!("{}image/process.{}?src={}", pictrs_url, file_type, name);

    if let Some(size) = params.max_size {
      url = format!("{url}&thumbnail={size}",);
    }
    url
  };

  do_get_image(processed_url, req, &context, None).await
}

pub async fn image_proxy(
  Query(params): Query<ImageProxyParams>,
  req: HttpRequest,
  context: Data<LemmyContext>,
) -> LemmyResult<Either<HttpResponse<()>, HttpResponse<BoxBody>>> {
  let url = Url::parse(&params.url)?;
  let encoded_url = utf8_percent_encode(&params.url, NON_ALPHANUMERIC).to_string();

  // Check that url corresponds to a federated image so that this can't be abused as a proxy
  // for arbitrary purposes.
  RemoteImage::validate(&mut context.pool(), url.clone().into()).await?;

  let pictrs_config = context.settings().pictrs()?;
  let output_file_type = (params.file_type.is_some() || params.max_size.is_some())
    .then(|| file_type(params.file_type.clone(), url.path()).unwrap_or_default());

  let processed_url = if let Some(file_type) = output_file_type {
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
    let download_filename = download_filename_from_url(url.path(), output_file_type);
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

  if let Some(disposition) = download_filename
    .as_deref()
    .and_then(inline_content_disposition)
  {
    client_res.insert_header((actix_web::http::header::CONTENT_DISPOSITION, disposition));
  }

  Ok(client_res.body(BodyStream::new(res.bytes_stream())))
}

#[derive(EnumString, Display, PartialEq, Debug, Default, Clone, Copy)]
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

/// Format a `Content-Disposition: inline` header value for the given filename.
fn inline_content_disposition(name: &str) -> Option<String> {
  let sanitized = sanitize_download_filename(name)?;

  if sanitized.is_ascii() {
    Some(format!("inline; filename=\"{}\"", sanitized))
  } else {
    // use filename* for non-ASCII names.
    // Also provide a plain ASCII fallback (non-ASCII chars replaced with '_')
    // for older clients that don't understand filename*.
    let ascii_fallback: String = sanitized
      .chars()
      .map(|c| if c.is_ascii() { c } else { '_' })
      .collect();
    let encoded = utf8_percent_encode(&sanitized, NON_ALPHANUMERIC).to_string();
    Some(format!(
      "inline; filename=\"{}\"; filename*=UTF-8''{}",
      ascii_fallback, encoded
    ))
  }
}

fn sanitize_download_filename(name: &str) -> Option<String> {
  let mut sanitized = name.to_string();
  sanitized.retain(|c| !is_unsafe_download_filename_char(c));

  if sanitized.is_empty() {
    None
  } else {
    Some(sanitized)
  }
}

fn is_unsafe_download_filename_char(c: char) -> bool {
  c.is_control()
    || matches!(
      c,
      '"'
        | '\\'
        | '/'
        | '\u{00AD}'
        | '\u{061C}'
        | '\u{180E}'
        | '\u{200B}'..='\u{200F}'
        | '\u{202A}'..='\u{202E}'
        | '\u{2060}'..='\u{2064}'
        | '\u{2066}'..='\u{206F}'
        | '\u{FEFF}'
    )
}

/// Extract the last path segment from a URL path string for use as a download filename
fn filename_from_url(url: &str) -> Option<String> {
  let raw = url
    .rsplit('/')
    .next()
    .filter(|s| !s.is_empty())
    // Strip any query-string that may be present
    .map(|s| s.split_once('?').map_or(s, |(before, _)| before))?;

  let decoded = percent_decode_str(raw).decode_utf8_lossy();
  sanitize_download_filename(decoded.as_ref())
}

fn download_filename_from_url(
  url: &str,
  output_file_type: Option<PictrsFileType>,
) -> Option<String> {
  let filename = filename_from_url(url)?;

  if let Some(file_type) = output_file_type {
    Some(filename_with_extension(&filename, file_type))
  } else {
    Some(filename)
  }
}

fn filename_with_extension(filename: &str, file_type: PictrsFileType) -> String {
  let stem = match filename.rsplit_once('.') {
    Some((stem, _)) if !stem.is_empty() => stem,
    _ => filename,
  };

  format!("{stem}.{}", file_type)
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
  use super::{
    PictrsFileType, download_filename_from_url, filename_from_url, inline_content_disposition,
    sanitize_download_filename,
  };
  use crate::images::download::file_type;
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
  fn test_filename_from_url() {
    // Simple path segment
    assert_eq!(
      filename_from_url("/media/photo.jpg"),
      Some("photo.jpg".to_string())
    );

    // pict-rs style UUID filename
    assert_eq!(
      filename_from_url("/pictrs/image/6d3b2f3f-7b29-4d9a-868e-b269423f4d6c.webp"),
      Some("6d3b2f3f-7b29-4d9a-868e-b269423f4d6c.webp".to_string())
    );

    // Query string is stripped
    assert_eq!(
      filename_from_url("/img.png?size=large"),
      Some("img.png".to_string())
    );

    // Percent-encoded spaces are decoded
    assert_eq!(
      filename_from_url("/media/holiday%20photo.jpg"),
      Some("holiday photo.jpg".to_string())
    );

    // Header-injection characters are stripped
    assert_eq!(
      filename_from_url("/media/evil\"inject.jpg"),
      Some("evilinject.jpg".to_string())
    );

    // Root-only path returns None
    assert_eq!(filename_from_url("/"), None);

    // Empty string input returns None
    assert_eq!(filename_from_url(""), None);
    // Backslash is stripped
    assert_eq!(
      filename_from_url("/media/evil\\inject.jpg"),
      Some("evilinject.jpg".to_string())
    );

    // Encoded slash is stripped from the suggested download name
    assert_eq!(
      filename_from_url("/media/evil%2Finject.jpg"),
      Some("evilinject.jpg".to_string())
    );

    // CR/LF injection characters are stripped
    assert_eq!(
      filename_from_url("/media/evil\rHeader\nInjected.jpg"),
      Some("evilHeaderInjected.jpg".to_string())
    );

    // Bidi override and zero-width characters are stripped to prevent spoofed names
    assert_eq!(
      filename_from_url("/images/evil%E2%80%AEgpj%E2%80%8B.exe"),
      Some("evilgpj.exe".to_string())
    );

    // Non-ASCII percent-encoded path decodes correctly
    assert_eq!(
      filename_from_url("/images/%C3%A9l%C3%A9phant.png"),
      Some("éléphant.png".to_string())
    );

    // Segment that becomes empty after stripping returns None
    assert_eq!(filename_from_url("/media/%22"), None);
  }

  #[test]
  fn test_sanitize_download_filename() {
    assert_eq!(
      sanitize_download_filename("safe file.jpg"),
      Some("safe file.jpg".to_string())
    );

    assert_eq!(
      sanitize_download_filename("evil\u{202E}gpj\u{200B}.exe"),
      Some("evilgpj.exe".to_string())
    );

    assert_eq!(sanitize_download_filename("\"\\/\r\n\u{202E}"), None);
  }

  #[test]
  fn test_download_filename_from_url() {
    assert_eq!(
      download_filename_from_url("/images/photo.png", Some(PictrsFileType::Avif)),
      Some("photo.avif".to_string())
    );

    assert_eq!(
      download_filename_from_url("/images/archive", Some(PictrsFileType::Webp)),
      Some("archive.webp".to_string())
    );

    assert_eq!(
      download_filename_from_url("/images/photo.tar.gz", Some(PictrsFileType::Jpg)),
      Some("photo.tar.jpg".to_string())
    );

    assert_eq!(
      download_filename_from_url("/images/%C3%A9l%C3%A9phant.png", Some(PictrsFileType::Jxl)),
      Some("éléphant.jxl".to_string())
    );

    assert_eq!(
      download_filename_from_url("/images/evil%E2%80%AEgpj.exe", Some(PictrsFileType::Webp)),
      Some("evilgpj.webp".to_string())
    );
  }

  #[test]
  fn test_inline_content_disposition() {
    // ASCII filename: simple quoted form
    assert_eq!(
      inline_content_disposition("photo.jpg"),
      Some("inline; filename=\"photo.jpg\"".to_string())
    );

    // ASCII UUID filename (as produced by pictrs)
    assert_eq!(
      inline_content_disposition("6d3b2f3f-7b29-4d9a-868e-b269423f4d6c.webp"),
      Some("inline; filename=\"6d3b2f3f-7b29-4d9a-868e-b269423f4d6c.webp\"".to_string())
    );

    // ASCII filename with spaces
    assert_eq!(
      inline_content_disposition("my photo.jpg"),
      Some("inline; filename=\"my photo.jpg\"".to_string())
    );

    // Non-ASCII filename: RFC 6266 dual form
    let result = inline_content_disposition("héron.jpg").expect("sanitized filename");
    // Must contain the ASCII fallback with underscores for the non-ASCII char
    assert!(result.contains("filename=\"h_ron.jpg\""), "got: {result}");
    // Must contain the UTF-8 encoded filename* parameter
    assert!(result.contains("filename*=UTF-8''"), "got: {result}");
    assert!(result.starts_with("inline; "), "got: {result}");

    // Fully non-ASCII name: fallback is all underscores
    let result2 = inline_content_disposition("写真.jpg").expect("sanitized filename");
    assert!(result2.contains("filename=\"__"), "got: {result2}");
    assert!(result2.contains(".jpg\""), "got: {result2}");
    assert!(result2.contains("filename*=UTF-8''"), "got: {result2}");

    // Direct callers also get hardened output.
    let hardened =
      inline_content_disposition("evil\u{202E}\r\nname.jpg").expect("sanitized filename");
    assert_eq!(hardened, "inline; filename=\"evilname.jpg\"");

    assert_eq!(inline_content_disposition("\"\\/\r\n\u{202E}"), None);
  }
}
