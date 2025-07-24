pub use crate::send::send_email_task;
use lemmy_db_schema::sensitive::SensitiveString;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use rosetta_i18n::{Language, LanguageId};
use std::sync::LazyLock;
use tokio::{
  sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender, WeakUnboundedSender},
    Mutex,
  },
  task::JoinHandle,
};
use translations::Lang;

pub mod account;
pub mod admin;
pub mod notifications;
mod send;

// TODO: this could be merged with `ActivityChannel` using generics
struct EmailChannel {
  weak_sender: WeakUnboundedSender<EmailParams>,
  receiver: Mutex<UnboundedReceiver<EmailParams>>,
  keepalive_sender: Mutex<Option<UnboundedSender<EmailParams>>>,
}

static EMAIL_CHANNEL: LazyLock<EmailChannel> = LazyLock::new(|| {
  let (sender, receiver) = unbounded_channel();
  let weak_sender = sender.downgrade();
  EmailChannel {
    weak_sender,
    receiver: Mutex::new(receiver),
    keepalive_sender: Mutex::new(Some(sender)),
  }
});

struct EmailParams {
  subject: String,
  to_email: SensitiveString,
  to_username: String,
  html: String,
  settings: &'static Settings,
}

pub async fn cancel_email_task(receive_task: JoinHandle<()>) -> LemmyResult<()> {
  EMAIL_CHANNEL.keepalive_sender.lock().await.take();
  receive_task.await?;
  Ok(())
}

/// Avoid warnings for unused 0.19 translations
#[allow(dead_code)]
mod translations {
  rosetta_i18n::include_translations!();
}

fn inbox_link(settings: &Settings) -> String {
  format!("{}/inbox", settings.get_protocol_and_hostname())
}

#[allow(clippy::expect_used)]
fn user_language(local_user_view: &LocalUserView) -> Lang {
  let lang_id = LanguageId::new(&local_user_view.local_user.interface_language);
  Lang::from_language_id(&lang_id).unwrap_or_else(|| {
    let en = LanguageId::new("en");
    Lang::from_language_id(&en).expect("default language")
  })
}

fn user_email(local_user_view: &LocalUserView) -> LemmyResult<SensitiveString> {
  local_user_view
    .local_user
    .email
    .clone()
    .ok_or(LemmyErrorType::EmailRequired.into())
}
