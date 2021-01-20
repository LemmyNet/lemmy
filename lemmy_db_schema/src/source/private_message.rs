use crate::schema::private_message;
use serde::Serialize;

#[derive(Clone, Queryable, Associations, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "private_message"]
pub struct PrivateMessage {
  pub id: i32,
  pub creator_id: i32,
  pub recipient_id: i32,
  pub content: String,
  pub deleted: bool,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub ap_id: String,
  pub local: bool,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "private_message"]
pub struct PrivateMessageForm {
  pub creator_id: i32,
  pub recipient_id: i32,
  pub content: String,
  pub deleted: Option<bool>,
  pub read: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub ap_id: Option<String>,
  pub local: bool,
}
