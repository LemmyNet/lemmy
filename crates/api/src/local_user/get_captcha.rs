use crate::captcha_as_wav_base64;
use actix_web::{
  http::{
    header::{CacheControl, CacheDirective},
    StatusCode,
  },
  web::{Data, Json},
  HttpResponse,
  HttpResponseBuilder,
};
use captcha::{gen, Difficulty};
use lemmy_api_common::{
  context::LemmyContext,
  person::{CaptchaResponse, GetCaptchaResponse},
  LemmyErrorType,
};
use lemmy_db_schema::source::captcha_answer::{CaptchaAnswer, CaptchaAnswerForm};
use lemmy_db_views::structs::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn get_captcha(context: Data<LemmyContext>) -> LemmyResult<HttpResponse> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  let mut res = HttpResponseBuilder::new(StatusCode::OK);
  res.insert_header(CacheControl(vec![CacheDirective::NoStore]));

  if !local_site.captcha_enabled {
    return Ok(res.json(Json(GetCaptchaResponse { ok: None })));
  }

  let captcha = gen(match local_site.captcha_difficulty.as_str() {
    "easy" => Difficulty::Easy,
    "hard" => Difficulty::Hard,
    _ => Difficulty::Medium,
  });

  let answer = captcha.chars_as_string();

  let png = captcha
    .as_base64()
    .ok_or(LemmyErrorType::CouldntCreateImageCaptcha)?;

  let wav = captcha_as_wav_base64(&captcha)?;

  let captcha_form: CaptchaAnswerForm = CaptchaAnswerForm { answer };
  // Stores the captcha item in the db
  let captcha = CaptchaAnswer::insert(&mut context.pool(), &captcha_form).await?;

  let json = Json(GetCaptchaResponse {
    ok: Some(CaptchaResponse {
      png,
      wav,
      uuid: captcha.uuid.to_string(),
    }),
  });
  Ok(res.json(json))
}
