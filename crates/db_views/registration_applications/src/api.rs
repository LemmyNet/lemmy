use crate::RegistrationApplicationView;
use lemmy_db_schema::newtypes::RegistrationApplicationId;
use lemmy_db_schema_file::PersonId;
use lemmy_diesel_utils::{pagination::PaginationCursor, sensitive::SensitiveString};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Approves a registration application.
pub struct ApproveRegistrationApplication {
  pub id: RegistrationApplicationId,
  pub approve: bool,
  pub deny_reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets a registration application for a person
pub struct GetRegistrationApplication {
  pub person_id: PersonId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Fetches a list of registration applications.
pub struct ListRegistrationApplications {
  /// Only shows the unread applications (IE those without an admin actor)
  pub unread_only: Option<bool>,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Register / Sign up to lemmy.
pub struct Register {
  pub username: String,
  pub password: SensitiveString,
  pub password_verify: SensitiveString,
  pub show_nsfw: Option<bool>,
  /// email is mandatory if email verification is enabled on the server
  pub email: Option<SensitiveString>,
  /// The UUID of the captcha item.
  pub captcha_uuid: Option<String>,
  /// Your captcha answer.
  pub captcha_answer: Option<String>,
  /// A form field to trick signup bots. Should be None.
  pub honeypot: Option<String>,
  /// An answer is mandatory if require application is enabled on the server
  pub answer: Option<String>,
  /// If this is true the login is valid forever, otherwise it expires after one week.
  pub stay_logged_in: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The response of an action done to a registration application.
pub struct RegistrationApplicationResponse {
  pub registration_application: RegistrationApplicationView,
}
