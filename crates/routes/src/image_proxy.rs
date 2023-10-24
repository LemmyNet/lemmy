use actix_web::{
  web,
  web::{Query, ServiceConfig},
  HttpResponse,
};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::images::RemoteImage;
use lemmy_utils::{error::LemmyResult, rate_limit::RateLimitCell};
use serde::Deserialize;
use url::Url;
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
  let url = Url::parse(&decode(&params.url)?)?;

  // Check that url corresponds to a federated image so that this can't be abused as a proxy
  // for arbitrary purposes.
  RemoteImage::validate(&mut context.pool(), url.clone().into()).await?;

  // TODO: Once pictrs 0.5 is out, use it for proxying like GET /image/original?proxy={url}
  //       https://git.asonix.dog/asonix/pict-rs/#api
  let image_response = context.client().get(url).send().await?;

  Ok(HttpResponse::Ok().streaming(image_response.bytes_stream()))
}
