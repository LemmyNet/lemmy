use super::utils::{adapt_request, convert_header};
use actix_web::{
  HttpRequest,
  HttpResponse,
  Responder,
  body::{BodyStream, BoxBody},
  http::StatusCode,
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

  do_get_image(processed_url, req, &context).await
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

  let pictrs_config = context.settings().pictrs()?;
  let processed_url = if params.file_type.is_none() && params.max_size.is_none() {
    format!("{}image/original?proxy={}", pictrs_config.url, encoded_url)
  } else {
    let file_type = file_type(params.file_type, url.path()).unwrap_or_default();

    let mut url = format!(
      "{}image/process.{}?proxy={}",
      pictrs_config.url, file_type, encoded_url
    );

    if let Some(size) = params.max_size {
      url = format!("{url}&thumbnail={size}",);
    }
    url
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
    Ok(Either::Right(
      do_get_image(processed_url, req, &context).await?,
    ))
  }
}

pub(super) async fn do_get_image(
  url: String,
  req: HttpRequest,
  context: &LemmyContext,
) -> LemmyResult<HttpResponse> {
  let mut client_req = adapt_request(&req, url, context);

  if let Some(addr) = req.head().peer_addr {
    client_req = client_req.header("X-Forwarded-For", addr.to_string());
  }

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

  Ok(client_res.body(BodyStream::new(res.bytes_stream())))
}

/// Auth required if instance is private with federation disabled
fn is_auth_required(local_user_view: Option<&LocalUserView>, local_site: &LocalSite) -> bool {
  local_user_view.is_none() && local_site.is_instance_private_federation_disabled()
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

/// Take file type from param, name, or use jpg if nothing is given
fn file_type(file_type: Option<String>, name: &str) -> LemmyResult<PictrsFileType> {
  let type_str = file_type
    .clone()
    .unwrap_or_else(|| name.split('.').next_back().unwrap_or("jpg").to_string());

  PictrsFileType::from_str(&type_str).with_lemmy_type(LemmyErrorType::NotAnImageType)
}

#[cfg(test)]
mod tests {
  use crate::images::download::{PictrsFileType, file_type};
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
}
