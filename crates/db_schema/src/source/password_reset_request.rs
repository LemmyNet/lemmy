use crate::schema::password_reset_request;

#[derive(Queryable, Identifiable, PartialEq, Debug)]
#[table_name = "password_reset_request"]
pub struct PasswordResetRequest {
  pub id: i32,
  pub local_user_id: i32,
  pub token_encrypted: String,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "password_reset_request"]
pub struct PasswordResetRequestForm {
  pub local_user_id: i32,
  pub token_encrypted: String,
}
