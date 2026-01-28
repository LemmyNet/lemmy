use actix_web::{
  HttpResponse,
  HttpResponseBuilder,
  http::{
    StatusCode,
    header::{CacheControl, CacheDirective},
  },
  web::{Data, Json},
};
use lemmy_api_utils::{context::LemmyContext, plugins::plugin_get_captcha};
use lemmy_db_views_site::{SiteView, api::GetCaptchaResponse};
use lemmy_utils::error::LemmyResult;

pub async fn get_captcha(context: Data<LemmyContext>) -> LemmyResult<HttpResponse> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  let mut res = HttpResponseBuilder::new(StatusCode::OK);
  res.insert_header(CacheControl(vec![CacheDirective::NoStore]));

  if !local_site.captcha_enabled {
    return Ok(res.json(Json(GetCaptchaResponse { ok: None })));
  }

  let captcha = GetCaptchaResponse {
    ok: Some(plugin_get_captcha().await?),
  };
  Ok(res.json(Json(captcha)))
}
