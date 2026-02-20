use lemmy_db_schema::source::local_user::LocalUser;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::sensitive::SensitiveString;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use rosetta_i18n::{Language, LanguageId};
use translations::Lang;

pub mod account;
pub mod admin;
pub mod notifications;
mod send;

/// Avoid warnings for unused 0.19 translations
#[allow(dead_code, mismatched_lifetime_syntaxes)]
pub mod translations {
  rosetta_i18n::include_translations!();
}

fn inbox_link(settings: &Settings) -> String {
  format!("{}/inbox", settings.get_protocol_and_hostname())
}

#[allow(clippy::expect_used)]
pub fn user_language(local_user: &LocalUser) -> Lang {
  let lang_id = LanguageId::new(&local_user.interface_language);
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
