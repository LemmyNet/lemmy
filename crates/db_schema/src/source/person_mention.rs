use crate::{schema::person_mention, source::comment::Comment};
use serde::Serialize;

#[derive(Clone, Queryable, Associations, Identifiable, PartialEq, Debug, Serialize)]
#[belongs_to(Comment)]
#[table_name = "person_mention"]
pub struct PersonMention {
  pub id: i32,
  pub recipient_id: i32,
  pub comment_id: i32,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "person_mention"]
pub struct PersonMentionForm {
  pub recipient_id: i32,
  pub comment_id: i32,
  pub read: Option<bool>,
}
