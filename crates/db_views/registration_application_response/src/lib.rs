use lemmy_db_views_registration_applications::RegistrationApplicationView;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response of an action done to a registration application.
pub struct RegistrationApplicationResponse {
  pub registration_application: RegistrationApplicationView,
}
