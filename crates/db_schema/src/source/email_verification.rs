use crate::{newtypes::LocalUserId, schema::email_verification};

#[derive(Queryable, Identifiable, Clone)]
#[table_name = "email_verification"]
pub struct EmailVerification {
  pub id: i32,
  pub local_user_id: LocalUserId,
  pub email: String,
  pub verification_code: String,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "email_verification"]
pub struct EmailVerificationForm {
  pub local_user_id: LocalUserId,
  pub email: String,
  pub verification_token: String,
}
