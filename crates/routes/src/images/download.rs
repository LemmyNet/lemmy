use super::utils::{adapt_request, convert_header};
use actix_web::{
  body::{BodyStream, BoxBody},
  http::StatusCode,
  web::{Data, *},
  HttpRequest,
  HttpResponse,
  Responder,
};
use lemmy_api_common::{
  context::LemmyContext,
  image::{ImageGetParams, ImageProxyParams},
};
use lemmy_db_schema::source::{images::RemoteImage, local_site::LocalSite};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;
use url::Url;

pub async fn get_image(
  filename: Path<String>,
  Query(params): Query<ImageGetParams>,
  req: HttpRequest,
  local_user_view: Option<LocalUserView>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  // block access to images if instance is private
  if local_user_view.is_none() {
    let local_site = LocalSite::read(&mut context.pool()).await?;
    if local_site.private_instance {
      return Ok(HttpResponse::Unauthorized().finish());
    }
  }
  let name = &filename.into_inner();

  // If there are no query params, the URL is original
  let pictrs_url = context.settings().pictrs()?.url;
  let processed_url = if params.file_type.is_none() && params.max_size.is_none() {
    format!("{}image/original/{}", pictrs_url, name)
  } else {
    let file_type = file_type(params.file_type, name);
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
  context: Data<LemmyContext>,
) -> LemmyResult<Either<HttpResponse<()>, HttpResponse<BoxBody>>> {
  let url = Url::parse(&params.url)?;

  // Check that url corresponds to a federated image so that this can't be abused as a proxy
  // for arbitrary purposes.
  RemoteImage::validate(&mut context.pool(), url.clone().into()).await?;

  let pictrs_config = context.settings().pictrs()?;
  let processed_url = if params.file_type.is_none() && params.max_size.is_none() {
    format!("{}image/original?proxy={}", pictrs_config.url, params.url)
  } else {
    let file_type = file_type(params.file_type, url.as_str());
    let mut url = format!(
      "{}image/process.{}?proxy={}",
      pictrs_config.url, file_type, url
    );

    if let Some(size) = params.max_size {
      url = format!("{url}&thumbnail={size}",);
    }
    url
  };

  let bypass_proxy = pictrs_config
    .proxy_bypass_domains
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

/// Take file type from param, name, or use jpg if nothing is given
pub(super) fn file_type(file_type: Option<String>, name: &str) -> String {
  file_type
    .clone()
    .unwrap_or_else(|| name.split('.').last().unwrap_or("jpg").to_string())
}
