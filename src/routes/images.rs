use actix::clock::Duration;
use actix_web::{body::BodyStream, http::StatusCode, *};
use awc::Client;
use lemmy_rate_limit::RateLimit;
use lemmy_utils::settings::Settings;
use serde::{Deserialize, Serialize};

pub fn config(cfg: &mut web::ServiceConfig, rate_limit: &RateLimit) {
  let client = Client::build()
    .header("User-Agent", "pict-rs-frontend, v0.1.0")
    .timeout(Duration::from_secs(30))
    .finish();

  cfg
    .data(client)
    .service(
      web::resource("/pictrs/image")
        .wrap(rate_limit.image())
        .route(web::post().to(upload)),
    )
    .service(web::resource("/pictrs/image/{filename}").route(web::get().to(full_res)))
    .service(
      web::resource("/pictrs/image/thumbnail{size}/{filename}").route(web::get().to(thumbnail)),
    )
    .service(web::resource("/pictrs/image/delete/{token}/{filename}").route(web::get().to(delete)));
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
  file: String,
  delete_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Images {
  msg: String,
  files: Option<Vec<Image>>,
}

async fn upload(
  req: HttpRequest,
  body: web::Payload,
  client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
  // TODO: check auth and rate limit here

  let mut res = client
    .request_from(format!("{}/image", Settings::get().pictrs_url), req.head())
    .if_some(req.head().peer_addr, |addr, req| {
      req.header("X-Forwarded-For", addr.to_string())
    })
    .send_stream(body)
    .await?;

  let images = res.json::<Images>().await?;

  Ok(HttpResponse::build(res.status()).json(images))
}

async fn full_res(
  filename: web::Path<String>,
  req: HttpRequest,
  client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
  let url = format!(
    "{}/image/{}",
    Settings::get().pictrs_url,
    &filename.into_inner()
  );
  image(url, req, client).await
}

async fn thumbnail(
  parts: web::Path<(u64, String)>,
  req: HttpRequest,
  client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
  let (size, file) = parts.into_inner();

  let url = format!(
    "{}/image/thumbnail{}/{}",
    Settings::get().pictrs_url,
    size,
    &file
  );

  image(url, req, client).await
}

async fn image(
  url: String,
  req: HttpRequest,
  client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
  let res = client
    .request_from(url, req.head())
    .if_some(req.head().peer_addr, |addr, req| {
      req.header("X-Forwarded-For", addr.to_string())
    })
    .no_decompress()
    .send()
    .await?;

  if res.status() == StatusCode::NOT_FOUND {
    return Ok(HttpResponse::NotFound().finish());
  }

  let mut client_res = HttpResponse::build(res.status());

  for (name, value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
    client_res.header(name.clone(), value.clone());
  }

  Ok(client_res.body(BodyStream::new(res)))
}

async fn delete(
  components: web::Path<(String, String)>,
  req: HttpRequest,
  client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
  let (token, file) = components.into_inner();

  let url = format!(
    "{}/image/delete/{}/{}",
    Settings::get().pictrs_url,
    &token,
    &file
  );
  let res = client
    .request_from(url, req.head())
    .if_some(req.head().peer_addr, |addr, req| {
      req.header("X-Forwarded-For", addr.to_string())
    })
    .no_decompress()
    .send()
    .await?;

  Ok(HttpResponse::build(res.status()).body(BodyStream::new(res)))
}
