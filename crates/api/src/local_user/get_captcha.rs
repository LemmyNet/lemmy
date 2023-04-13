use crate::{captcha_as_wav_base64, Perform};
use actix_web::web::Data;
use captcha::{gen, Difficulty};
use chrono::Duration;
use lemmy_api_common::{
  context::LemmyContext,
  person::{CaptchaResponse, GetCaptcha, GetCaptchaResponse},
  websocket::{handlers::captcha::AddCaptcha, structs::CaptchaItem},
};
use lemmy_db_schema::{source::local_site::LocalSite, utils::naive_now};
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for GetCaptcha {
  type Response = GetCaptchaResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let local_site = LocalSite::read(context.pool()).await?;

    if !local_site.captcha_enabled {
      return Ok(GetCaptchaResponse { ok: None });
    }

    let captcha = gen(match local_site.captcha_difficulty.as_str() {
      "easy" => Difficulty::Easy,
      "hard" => Difficulty::Hard,
      _ => Difficulty::Medium,
    });

    let answer = captcha.chars_as_string();

    let png = captcha.as_base64().expect("failed to generate captcha");

    let uuid = uuid::Uuid::new_v4().to_string();

    let wav = captcha_as_wav_base64(&captcha);

    let captcha_item = CaptchaItem {
      answer,
      uuid: uuid.clone(),
      expires: naive_now() + Duration::minutes(10), // expires in 10 minutes
    };

    // Stores the captcha item on the queue
    context.chat_server().do_send(AddCaptcha {
      captcha: captcha_item,
    });

    Ok(GetCaptchaResponse {
      ok: Some(CaptchaResponse { png, wav, uuid }),
    })
  }
}
