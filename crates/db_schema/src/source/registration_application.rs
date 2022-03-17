use crate::{
  newtypes::{LocalUserId, PersonId},
  schema::registration_application,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
#[table_name = "registration_application"]
pub struct RegistrationApplication {
  pub id: i32,
  pub local_user_id: LocalUserId,
  pub answer: String,
  pub admin_id: Option<PersonId>,
  pub deny_reason: Option<String>,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Default)]
#[table_name = "registration_application"]
pub struct RegistrationApplicationForm {
  pub local_user_id: Option<LocalUserId>,
  pub answer: Option<String>,
  pub admin_id: Option<PersonId>,
  pub deny_reason: Option<Option<String>>,
}
