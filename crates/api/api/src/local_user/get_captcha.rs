use crate::captcha_as_wav_base64;
use actix_web::{
  HttpResponse,
  HttpResponseBuilder,
  http::{
    StatusCode,
    header::{CacheControl, CacheDirective},
  },
  web::{Data, Json},
};
use captcha::{Difficulty, generate};
use lemmy_api_utils::{context::LemmyContext, plugins::plugin_get_captcha};
use lemmy_db_schema::source::captcha_answer::{CaptchaAnswer, CaptchaAnswerForm};
use lemmy_db_views_site::{
  SiteView,
  api::{CaptchaResponse, GetCaptchaResponse},
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn get_captcha(context: Data<LemmyContext>) -> LemmyResult<HttpResponse> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  let mut res = HttpResponseBuilder::new(StatusCode::OK);
  res.insert_header(CacheControl(vec![CacheDirective::NoStore]));

  if !local_site.captcha_enabled {
    return Ok(res.json(Json(GetCaptchaResponse { ok: None })));
  }

  let captcha = plugin_get_captcha().await?;
  Ok(res.json(Json(captcha)))
}
