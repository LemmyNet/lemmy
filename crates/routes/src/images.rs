use actix_web::{
  body::BodyStream,
  error,
  http::{
    header::{HeaderName, ACCEPT_ENCODING, HOST},
    StatusCode,
  },
  web,
  Error,
  HttpRequest,
  HttpResponse,
};
use futures::stream::{Stream, StreamExt};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::{
  images::{LocalImage, LocalImageForm},
  local_site::LocalSite,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{rate_limit::RateLimitCell, REQWEST_TIMEOUT};
use reqwest::Body;
use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub fn config(
  cfg: &mut web::ServiceConfig,
  client: ClientWithMiddleware,
  rate_limit: &RateLimitCell,
) {
  cfg
    .app_data(web::Data::new(client))
    .service(
      web::resource("/pictrs/image")
        .wrap(rate_limit.image())
        .route(web::post().to(upload)),
    )
    // This has optional query params: /image/{filename}?format=jpg&thumbnail=256
    .service(web::resource("/pictrs/image/{filename}").route(web::get().to(full_res)))
    .service(web::resource("/pictrs/image/delete/{token}/{filename}").route(web::get().to(delete)));
}

#[derive(Debug, Serialize, Deserialize)]
struct Image {
  file: String,
  delete_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Images {
  msg: String,
  files: Option<Vec<Image>>,
}

#[derive(Deserialize)]
struct PictrsParams {
  format: Option<String>,
  thumbnail: Option<i32>,
}

#[derive(Deserialize)]
enum PictrsPurgeParams {
  #[serde(rename = "file")]
  File(String),
  #[serde(rename = "alias")]
  Alias(String),
}

fn adapt_request(
  request: &HttpRequest,
  client: &ClientWithMiddleware,
  url: String,
) -> RequestBuilder {
  // remove accept-encoding header so that pictrs doesnt compress the response
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
  body: web::Payload,
  // require login
  local_user_view: LocalUserView,
  client: web::Data<ClientWithMiddleware>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  // TODO: check rate limit here
  let pictrs_config = context.settings().pictrs_config()?;
  let image_url = format!("{}image", pictrs_config.url);

  let mut client_req = adapt_request(&req, &client, image_url);

  if let Some(addr) = req.head().peer_addr {
    client_req = client_req.header("X-Forwarded-For", addr.to_string())
  };
  let res = client_req
    .timeout(Duration::from_secs(30))
    .body(Body::wrap_stream(make_send(body)))
    .send()
    .await
    .map_err(error::ErrorBadRequest)?;

  let status = res.status();
  let images = res.json::<Images>().await.map_err(error::ErrorBadRequest)?;
  if let Some(images) = &images.files {
    for uploaded_image in images {
      let form = LocalImageForm {
        local_user_id: local_user_view.local_user.id,
        pictrs_alias: uploaded_image.file.to_string(),
        pictrs_delete_token: uploaded_image.delete_token.to_string(),
      };
      LocalImage::create(&mut context.pool(), &form)
        .await
        .map_err(error::ErrorBadRequest)?;
    }
  }

  Ok(HttpResponse::build(status).json(images))
}

async fn full_res(
  filename: web::Path<String>,
  web::Query(params): web::Query<PictrsParams>,
  req: HttpRequest,
  client: web::Data<ClientWithMiddleware>,
  context: web::Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> Result<HttpResponse, Error> {
  // block access to images if instance is private and unauthorized, public
  let local_site = LocalSite::read(&mut context.pool())
    .await
    .map_err(error::ErrorBadRequest)?;
  if local_site.private_instance && local_user_view.is_none() {
    return Ok(HttpResponse::Unauthorized().finish());
  }
  let name = &filename.into_inner();

  // If there are no query params, the URL is original
  let pictrs_config = context.settings().pictrs_config()?;
  let url = if params.format.is_none() && params.thumbnail.is_none() {
    format!("{}image/original/{}", pictrs_config.url, name,)
  } else {
    // Take file type from name, or jpg if nothing is given
    let format = params
      .format
      .unwrap_or_else(|| name.split('.').last().unwrap_or("jpg").to_string());

    let mut url = format!("{}image/process.{}?src={}", pictrs_config.url, format, name,);

    if let Some(size) = params.thumbnail {
      url = format!("{url}&thumbnail={size}",);
    }
    url
  };

  image(url, req, &client).await
}

async fn image(
  url: String,
  req: HttpRequest,
  client: &ClientWithMiddleware,
) -> Result<HttpResponse, Error> {
  let mut client_req = adapt_request(&req, client, url);

  if let Some(addr) = req.head().peer_addr {
    client_req = client_req.header("X-Forwarded-For", addr.to_string());
  }

  if let Some(addr) = req.head().peer_addr {
    client_req = client_req.header("X-Forwarded-For", addr.to_string());
  }

  let res = client_req.send().await.map_err(error::ErrorBadRequest)?;

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
  components: web::Path<(String, String)>,
  req: HttpRequest,
  client: web::Data<ClientWithMiddleware>,
  context: web::Data<LemmyContext>,
  // require login
  _local_user_view: LocalUserView,
) -> Result<HttpResponse, Error> {
  let (token, file) = components.into_inner();

  let pictrs_config = context.settings().pictrs_config()?;
  let url = format!("{}image/delete/{}/{}", pictrs_config.url, &token, &file);

  let mut client_req = adapt_request(&req, &client, url);

  if let Some(addr) = req.head().peer_addr {
    client_req = client_req.header("X-Forwarded-For", addr.to_string());
  }

  let res = client_req.send().await.map_err(error::ErrorBadRequest)?;

  LocalImage::delete_by_alias(&mut context.pool(), &file)
    .await
    .map_err(error::ErrorBadRequest)?;

  Ok(HttpResponse::build(res.status()).body(BodyStream::new(res.bytes_stream())))
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
