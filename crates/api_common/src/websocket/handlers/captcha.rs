use crate::websocket::{chat_server::ChatServer, structs::CaptchaItem};
use actix::{Context, Handler, Message};
use lemmy_db_schema::utils::naive_now;

/// Adding a Captcha
#[derive(Message)]
#[rtype(result = "()")]
pub struct AddCaptcha {
  pub captcha: CaptchaItem,
}

impl Handler<AddCaptcha> for ChatServer {
  type Result = ();

  fn handle(&mut self, msg: AddCaptcha, _: &mut Context<Self>) -> Self::Result {
    self.captchas.push(msg.captcha);
  }
}

/// Checking a Captcha
#[derive(Message)]
#[rtype(bool)]
pub struct CheckCaptcha {
  pub uuid: String,
  pub answer: String,
}

impl Handler<CheckCaptcha> for ChatServer {
  type Result = bool;

  fn handle(&mut self, msg: CheckCaptcha, _: &mut Context<Self>) -> Self::Result {
    // Remove all the ones that are past the expire time
    self.captchas.retain(|x| x.expires.gt(&naive_now()));

    let check = self
      .captchas
      .iter()
      .any(|r| r.uuid == msg.uuid && r.answer.to_lowercase() == msg.answer.to_lowercase());

    // Remove this uuid so it can't be re-checked (Checks only work once)
    self.captchas.retain(|x| x.uuid != msg.uuid);

    check
  }
}
