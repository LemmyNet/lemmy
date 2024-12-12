use actix_web::{
  body::{BodyStream, BoxBody},
  http::{
    header::{HeaderName, ACCEPT_ENCODING, HOST},
    StatusCode,
  },
  web::*,
  HttpRequest,
  HttpResponse,
  Responder,
};
use lemmy_api_common::{context::LemmyContext, request::PictrsResponse};
use lemmy_db_schema::source::{
  images::{LocalImage, LocalImageForm, RemoteImage},
  local_site::LocalSite,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{error::LemmyResult, REQWEST_TIMEOUT};
use reqwest::Body;
use reqwest_middleware::RequestBuilder;
use serde::Deserialize;
use std::time::Duration;
use url::Url;
use utils::{convert_header, convert_method, convert_status, make_send};

mod utils;

trait ProcessUrl {
  /// If thumbnail or format is given, this uses the pictrs process endpoint.
  /// Otherwise, it uses the normal pictrs url (IE image/original).
  fn process_url(&self, image_url: &str, pictrs_url: &Url) -> String;
}

#[derive(Deserialize, Clone)]
pub struct PictrsGetParams {
  format: Option<String>,
  thumbnail: Option<i32>,
}

impl ProcessUrl for PictrsGetParams {
  fn process_url(&self, src: &str, pictrs_url: &Url) -> String {
    if self.format.is_none() && self.thumbnail.is_none() {
      format!("{}image/original/{}", pictrs_url, src)
    } else {
      // Take file type from name, or jpg if nothing is given
      let format = self
        .clone()
        .format
        .unwrap_or_else(|| src.split('.').last().unwrap_or("jpg").to_string());

      let mut url = format!("{}image/process.{}?src={}", pictrs_url, format, src);

      if let Some(size) = self.thumbnail {
        url = format!("{url}&thumbnail={size}",);
      }
      url
    }
  }
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
fn adapt_request(request: &HttpRequest, context: &LemmyContext, url: String) -> RequestBuilder {
  // remove accept-encoding header so that pictrs doesn't compress the response
  const INVALID_HEADERS: &[HeaderName] = &[ACCEPT_ENCODING, HOST];

  let client_request = context
    .client()
    .request(convert_method(request.method()), url)
    .timeout(REQWEST_TIMEOUT);

  request
    .headers()
    .iter()
    .fold(client_request, |client_req, (key, value)| {
      if INVALID_HEADERS.contains(key) {
        client_req
      } else {
        // TODO: remove as_str and as_bytes conversions after actix-web upgrades to http 1.0
        client_req.header(key.as_str(), value.as_bytes())
      }
    })
}

pub async fn upload_image(
  req: HttpRequest,
  body: Payload,
  // require login
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let pictrs_config = context.settings().pictrs_config()?;
  let image_url = format!("{}image", pictrs_config.url);

  let mut client_req = adapt_request(&req, &context, image_url);

  if let Some(addr) = req.head().peer_addr {
    client_req = client_req.header("X-Forwarded-For", addr.to_string())
  };
  let res = client_req
    .timeout(Duration::from_secs(pictrs_config.upload_timeout))
    .body(Body::wrap_stream(make_send(body)))
    .send()
    .await?;

  let status = res.status();
  let images = res.json::<PictrsResponse>().await?;
  if let Some(images) = &images.files {
    for image in images {
      let form = LocalImageForm {
        local_user_id: Some(local_user_view.local_user.id),
        pictrs_alias: image.file.to_string(),
        pictrs_delete_token: image.delete_token.to_string(),
      };

      let protocol_and_hostname = context.settings().get_protocol_and_hostname();
      let thumbnail_url = image.thumbnail_url(&protocol_and_hostname)?;

      // Also store the details for the image
      let details_form = image.details.build_image_details_form(&thumbnail_url);
      LocalImage::create(&mut context.pool(), &form, &details_form).await?;
    }
  }

  Ok(HttpResponse::build(convert_status(status)).json(images))
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

  image(processed_url, req, &context).await
}

async fn image(url: String, req: HttpRequest, context: &LemmyContext) -> LemmyResult<HttpResponse> {
  let mut client_req = adapt_request(&req, context, url);

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

pub async fn delete_image(
  components: Path<(String, String)>,
  req: HttpRequest,
  context: Data<LemmyContext>,
  // require login
  _local_user_view: LocalUserView,
) -> LemmyResult<HttpResponse> {
  let (token, file) = components.into_inner();

  let pictrs_config = context.settings().pictrs_config()?;
  let url = format!("{}image/delete/{}/{}", pictrs_config.url, &token, &file);

  let mut client_req = adapt_request(&req, &context, url);

  if let Some(addr) = req.head().peer_addr {
    client_req = client_req.header("X-Forwarded-For", addr.to_string());
  }

  let res = client_req.send().await?;

  LocalImage::delete_by_alias(&mut context.pool(), &file).await?;

  Ok(HttpResponse::build(convert_status(res.status())).body(BodyStream::new(res.bytes_stream())))
}

pub async fn pictrs_healthz(
  req: HttpRequest,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let pictrs_config = context.settings().pictrs_config()?;
  let url = format!("{}healthz", pictrs_config.url);

  let mut client_req = adapt_request(&req, &context, url);

  if let Some(addr) = req.head().peer_addr {
    client_req = client_req.header("X-Forwarded-For", addr.to_string());
  }

  let res = client_req.send().await?;

  Ok(HttpResponse::build(convert_status(res.status())).body(BodyStream::new(res.bytes_stream())))
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
    Ok(Either::Right(image(processed_url, req, &context).await?))
  }
}
