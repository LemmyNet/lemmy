use crate::newtypes::{LocalUserId, PersonId};
use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
use crate::schema::registration_application;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = registration_application))]
pub struct RegistrationApplication {
  pub id: i32,
  pub local_user_id: LocalUserId,
  pub answer: String,
  pub admin_id: Option<PersonId>,
  pub deny_reason: Option<String>,
  pub published: chrono::NaiveDateTime,
}

#[derive(Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = registration_application))]
pub struct RegistrationApplicationForm {
  pub local_user_id: Option<LocalUserId>,
  pub answer: Option<String>,
  pub admin_id: Option<PersonId>,
  pub deny_reason: Option<Option<String>>,
}
