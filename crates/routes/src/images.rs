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
use futures::stream::{Stream, StreamExt};
use lemmy_api_common::{
  context::LemmyContext,
  request::{delete_image_from_pictrs, PictrsResponse},
};
use lemmy_db_schema::source::{
  images::{LocalImage, LocalImageForm, RemoteImage},
  local_site::LocalSite,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{error::LemmyResult, rate_limit::RateLimitCell, REQWEST_TIMEOUT};
use reqwest::Body;
use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
use serde::Deserialize;
use std::time::Duration;
use url::Url;

pub fn config(cfg: &mut ServiceConfig, client: ClientWithMiddleware, rate_limit: &RateLimitCell) {
  cfg
    .app_data(Data::new(client))
    .service(
      resource("/pictrs/image")
        .wrap(rate_limit.image())
        .route(post().to(upload)),
    )
    // This has optional query params: /image/{filename}?format=jpg&thumbnail=256
    .service(resource("/pictrs/image/{filename}").route(get().to(full_res)))
    .service(resource("/pictrs/image/delete/{token}/{filename}").route(get().to(delete)))
    .service(resource("/pictrs/healthz").route(get().to(healthz)));
}

trait ProcessUrl {
  /// If thumbnail or format is given, this uses the pictrs process endpoint.
  /// Otherwise, it uses the normal pictrs url (IE image/original).
  fn process_url(&self, image_url: &str, pictrs_url: &Url) -> String;
}

#[derive(Deserialize, Clone)]
struct PictrsGetParams {
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
fn adapt_request(
  request: &HttpRequest,
  client: &ClientWithMiddleware,
  url: String,
) -> RequestBuilder {
  // remove accept-encoding header so that pictrs doesn't compress the response
  const INVALID_HEADERS: &[HeaderName] = &[ACCEPT_ENCODING, HOST];

  let client_request = client
    .request(request.method().clone(), url)
    .timeout(REQWEST_TIMEOUT);

  request
    .headers()
    .iter()
    .fold(client_request, |client_req, (key, value)| {
      if INVALID_HEADERS.contains(key) {
        client_req
      } else {
        client_req.header(key, value)
      }
    })
}

async fn upload(
  req: HttpRequest,
  body: Payload,
  // require login
  local_user_view: LocalUserView,
  client: Data<ClientWithMiddleware>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  // TODO: check rate limit here
  let pictrs_config = context.settings().pictrs_config()?;
  let image_url = format!("{}image", pictrs_config.url);

  let mut client_req = adapt_request(&req, &client, image_url);

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

  Ok(HttpResponse::build(status).json(images))
}

async fn full_res(
  filename: Path<String>,
  Query(params): Query<PictrsGetParams>,
  req: HttpRequest,
  client: Data<ClientWithMiddleware>,
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

  image(processed_url, req, &client).await
}

async fn image(
  url: String,
  req: HttpRequest,
  client: &ClientWithMiddleware,
) -> LemmyResult<HttpResponse> {
  let mut client_req = adapt_request(&req, client, url);

  if let Some(addr) = req.head().peer_addr {
    client_req = client_req.header("X-Forwarded-For", addr.to_string());
  }

  if let Some(addr) = req.head().peer_addr {
    client_req = client_req.header("X-Forwarded-For", addr.to_string());
  }

  let res = client_req.send().await?;

  if res.status() == StatusCode::NOT_FOUND {
    return Ok(HttpResponse::NotFound().finish());
  }

  let mut client_res = HttpResponse::build(res.status());

  for (name, value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
    client_res.insert_header((name.clone(), value.clone()));
  }

  Ok(client_res.body(BodyStream::new(res.bytes_stream())))
}

async fn delete(
  components: Path<(String, String)>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<HttpResponse> {
  // Deletion token is accepted for backwards API compatibility but no longer used for validation or
  // performing the deletion. It is not necessary to validate the token, as we validate that the
  // user is authorized by being an admin or by matching the local_user_id of the local_image to
  // their local_user.id.
  let (_token, alias) = components.into_inner();

  let image =
    LocalImage::delete_by_alias_and_user(&mut context.pool(), &local_user_view.local_user, &alias)
      .await?;

  delete_image_from_pictrs(&image.pictrs_alias, &image.pictrs_delete_token, &context).await?;

  // This replicates older behavior from when we passed through the pict-rs API response to avoid a
  // breaking change. See https://git.asonix.dog/asonix/pict-rs/src/commit/38d5f185775837ad0203d5006c65ec201d3109fa/src/lib.rs#L788
  Ok(HttpResponse::NoContent().finish())
}

async fn healthz(
  req: HttpRequest,
  client: Data<ClientWithMiddleware>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let pictrs_config = context.settings().pictrs_config()?;
  let url = format!("{}healthz", pictrs_config.url);

  let mut client_req = adapt_request(&req, &client, url);

  if let Some(addr) = req.head().peer_addr {
    client_req = client_req.header("X-Forwarded-For", addr.to_string());
  }

  let res = client_req.send().await?;

  Ok(HttpResponse::build(res.status()).body(BodyStream::new(res.bytes_stream())))
}

pub async fn image_proxy(
  Query(params): Query<ImageProxyParams>,
  req: HttpRequest,
  client: Data<ClientWithMiddleware>,
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
    Ok(Either::Right(image(processed_url, req, &client).await?))
  }
}

fn make_send<S>(mut stream: S) -> impl Stream<Item = S::Item> + Send + Unpin + 'static
where
  S: Stream + Unpin + 'static,
  S::Item: Send,
{
  // NOTE: the 8 here is arbitrary
  let (tx, rx) = tokio::sync::mpsc::channel(8);

  // NOTE: spawning stream into a new task can potentially hit this bug:
  // - https://github.com/actix/actix-web/issues/1679
  //
  // Since 4.0.0-beta.2 this issue is incredibly less frequent. I have not personally reproduced it.
  // That said, it is still technically possible to encounter.
  actix_web::rt::spawn(async move {
    while let Some(res) = stream.next().await {
      if tx.send(res).await.is_err() {
        break;
      }
    }
  });

  SendStream { rx }
}

struct SendStream<T> {
  rx: tokio::sync::mpsc::Receiver<T>,
}

impl<T> Stream for SendStream<T>
where
  T: Send,
{
  type Item = T;

  fn poll_next(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Self::Item>> {
    std::pin::Pin::new(&mut self.rx).poll_recv(cx)
  }
}
