use crate::newtypes::LocalUserId;
use chrono::{DateTime, Utc};
use derive_aliases::derive;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::email_verification;

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(..SqlStruct))]
#[cfg_attr(feature = "full", diesel(table_name = email_verification))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct EmailVerification {
  pub id: i32,
  pub local_user_id: LocalUserId,
  pub email: String,
  pub verification_token: String,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = email_verification))]
pub struct EmailVerificationForm {
  pub local_user_id: LocalUserId,
  pub email: String,
  pub verification_token: String,
}
