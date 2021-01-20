use crate::{schema::user_mention, source::comment::Comment};
use serde::Serialize;

#[derive(Clone, Queryable, Associations, Identifiable, PartialEq, Debug, Serialize)]
#[belongs_to(Comment)]
#[table_name = "user_mention"]
pub struct UserMention {
  pub id: i32,
  pub recipient_id: i32,
  pub comment_id: i32,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "user_mention"]
pub struct UserMentionForm {
  pub recipient_id: i32,
  pub comment_id: i32,
  pub read: Option<bool>,
}
