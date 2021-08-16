use actix_web::{error::ErrorBadRequest, web::Query, *};
use anyhow::anyhow;
use lemmy_utils::{request::fetch_post_link_tags, LemmyError};
use lemmy_websocket::LemmyContext;
use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
struct Params {
  url: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
  cfg.route("/post_link_tags", web::get().to(get_post_links_response));
}

async fn get_post_links_response(
  info: Query<Params>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error> {
  let url =
    Url::parse(&info.url).map_err(|_| ErrorBadRequest(LemmyError::from(anyhow!("not_found"))))?;

  let json = fetch_post_link_tags(context.client(), &url)
    .await
    .map_err(|_| ErrorBadRequest(LemmyError::from(anyhow!("not_found"))))?;

  Ok(HttpResponse::Ok().json(json))
}
