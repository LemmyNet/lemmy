use crate::RegistrationApplicationView;
use lemmy_db_schema::{
  newtypes::{PaginationCursor, PersonId, RegistrationApplicationId},
  sensitive::SensitiveString,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Approves a registration application.
pub struct ApproveRegistrationApplication {
  pub id: RegistrationApplicationId,
  pub approve: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub deny_reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Gets a registration application for a person
pub struct GetRegistrationApplication {
  pub person_id: PersonId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Fetches a list of registration applications.
pub struct ListRegistrationApplications {
  /// Only shows the unread applications (IE those without an admin actor)
  #[cfg_attr(feature = "full", ts(optional))]
  pub unread_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_cursor: Option<PaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_back: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The list of registration applications.
pub struct ListRegistrationApplicationsResponse {
  pub registration_applications: Vec<RegistrationApplicationView>,
  /// the pagination cursor to use to fetch the next page
  #[cfg_attr(feature = "full", ts(optional))]
  pub next_page: Option<PaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub prev_page: Option<PaginationCursor>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Register / Sign up to lemmy.
pub struct Register {
  pub username: String,
  pub password: SensitiveString,
  pub password_verify: SensitiveString,
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_nsfw: Option<bool>,
  /// email is mandatory if email verification is enabled on the server
  #[cfg_attr(feature = "full", ts(optional))]
  pub email: Option<SensitiveString>,
  /// The UUID of the captcha item.
  #[cfg_attr(feature = "full", ts(optional))]
  pub captcha_uuid: Option<String>,
  /// Your captcha answer.
  #[cfg_attr(feature = "full", ts(optional))]
  pub captcha_answer: Option<String>,
  /// A form field to trick signup bots. Should be None.
  #[cfg_attr(feature = "full", ts(optional))]
  pub honeypot: Option<String>,
  /// An answer is mandatory if require application is enabled on the server
  #[cfg_attr(feature = "full", ts(optional))]
  pub answer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response of an action done to a registration application.
pub struct RegistrationApplicationResponse {
  pub registration_application: RegistrationApplicationView,
}
