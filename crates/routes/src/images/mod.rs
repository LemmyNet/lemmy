use actix_web::{
  body::{BodyStream, BoxBody},
  http::StatusCode,
  web::*,
  HttpRequest,
  HttpResponse,
  Responder,
};
use lemmy_api_common::{context::LemmyContext, SuccessResponse};
use lemmy_db_schema::source::{
  images::{LocalImage, RemoteImage},
  local_site::LocalSite,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;
use serde::Deserialize;
use url::Url;
use utils::{
  adapt_request,
  convert_header,
  do_upload_image,
  PictrsGetParams,
  ProcessUrl,
  UploadType,
  PICTRS_CLIENT,
};

pub mod person;
mod utils;

pub async fn upload_image(
  req: HttpRequest,
  body: Payload,
  // require login
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let image = do_upload_image(req, body, UploadType::Other, &local_user_view, &context).await?;

  Ok(HttpResponse::Ok().json(image))
}

pub async fn get_full_res_image(
  filename: Path<String>,
  Query(params): Query<PictrsGetParams>,
  req: HttpRequest,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<HttpResponse> {
  // block access to images if instance is private and unauthorized, public
  let local_site = LocalSite::read(&mut context.pool()).await?;
  if local_site.private_instance && local_user_view.is_none() {
    return Ok(HttpResponse::Unauthorized().finish());
  }
  let name = &filename.into_inner();

  // If there are no query params, the URL is original
  let pictrs_config = context.settings().pictrs_config()?;

  let processed_url = params.process_url(name, &pictrs_config.url);

  image(processed_url, req).await
}

async fn image(url: String, req: HttpRequest) -> LemmyResult<HttpResponse> {
  let mut client_req = adapt_request(&req, url);

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

#[derive(Deserialize, Clone)]
pub struct DeleteImageParams {
  file: String,
  token: String,
}

pub async fn delete_image(
  data: Json<DeleteImageParams>,
  context: Data<LemmyContext>,
  // require login
  _local_user_view: LocalUserView,
) -> LemmyResult<SuccessResponse> {
  let pictrs_config = context.settings().pictrs_config()?;
  let url = format!(
    "{}image/delete/{}/{}",
    pictrs_config.url, &data.token, &data.file
  );

  PICTRS_CLIENT.delete(url).send().await?.error_for_status()?;

  LocalImage::delete_by_alias(&mut context.pool(), &data.file).await?;

  Ok(SuccessResponse::default())
}

pub async fn pictrs_healthz(context: Data<LemmyContext>) -> LemmyResult<SuccessResponse> {
  let pictrs_config = context.settings().pictrs_config()?;
  let url = format!("{}healthz", pictrs_config.url);

  PICTRS_CLIENT.get(url).send().await?.error_for_status()?;

  Ok(SuccessResponse::default())
}

#[derive(Deserialize, Clone)]
pub struct ImageProxyParams {
  url: String,
  format: Option<String>,
  thumbnail: Option<i32>,
}

impl ProcessUrl for ImageProxyParams {
  fn process_url(&self, proxy_url: &str, pictrs_url: &Url) -> String {
    if self.format.is_none() && self.thumbnail.is_none() {
      format!("{}image/original?proxy={}", pictrs_url, proxy_url)
    } else {
      // Take file type from name, or jpg if nothing is given
      let format = self
        .clone()
        .format
        .unwrap_or_else(|| proxy_url.split('.').last().unwrap_or("jpg").to_string());

      let mut url = format!("{}image/process.{}?proxy={}", pictrs_url, format, proxy_url);

      if let Some(size) = self.thumbnail {
        url = format!("{url}&thumbnail={size}",);
      }
      url
    }
  }
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

  let pictrs_config = context.settings().pictrs_config()?;
  let processed_url = params.process_url(&params.url, &pictrs_config.url);

  let bypass_proxy = pictrs_config
    .proxy_bypass_domains
    .iter()
    .any(|s| url.domain().is_some_and(|d| d == s));
  if bypass_proxy {
    // Bypass proxy and redirect user to original image
    Ok(Either::Left(Redirect::to(url.to_string()).respond_to(&req)))
  } else {
    // Proxy the image data through Lemmy
    Ok(Either::Right(image(processed_url, req).await?))
  }
}
