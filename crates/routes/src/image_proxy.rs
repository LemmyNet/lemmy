use actix_web::{
  web,
  web::{Query, ServiceConfig},
  HttpResponse,
};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::{error::LemmyResult, rate_limit::RateLimitCell};
use serde::Deserialize;
use urlencoding::decode;

pub fn config(cfg: &mut ServiceConfig, rate_limit: &RateLimitCell) {
  cfg.service(
    web::resource("/api/v3/image_proxy")
      .wrap(rate_limit.message())
      .route(web::post().to(image_proxy)),
  );
}

#[derive(Deserialize)]
struct ImageProxyParams {
  url: String,
}

async fn image_proxy(
  Query(params): Query<ImageProxyParams>,
  context: web::Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let url = decode(&params.url)?.into_owned();
  let image_response = context.client().get(url).send().await?;

  Ok(HttpResponse::Ok().streaming(image_response.bytes_stream()))
}
