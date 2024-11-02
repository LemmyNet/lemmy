use crate::newtypes::LocalUserId;
#[cfg(feature = "full")]
use crate::schema::email_verification;
use chrono::{DateTime, Utc};

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = email_verification))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct EmailVerification {
  pub id: i32,
  pub local_user_id: LocalUserId,
  pub email: String,
  pub verification_token: String,
  pub published: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = email_verification))]
pub struct EmailVerificationForm {
  pub local_user_id: LocalUserId,
  pub email: String,
  pub verification_token: String,
}
