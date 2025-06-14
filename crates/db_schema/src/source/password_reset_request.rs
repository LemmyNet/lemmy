use crate::{newtypes::LocalUserId, sensitive::SensitiveString};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::password_reset_request;

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = password_reset_request))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct PasswordResetRequest {
  pub id: i32,
  pub token: SensitiveString,
  pub published_at: DateTime<Utc>,
  pub local_user_id: LocalUserId,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = password_reset_request))]
pub struct PasswordResetRequestForm {
  pub local_user_id: LocalUserId,
  pub token: SensitiveString,
}
