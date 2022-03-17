use crate::{
  newtypes::{CommentId, PersonId, PersonMentionId},
  schema::person_mention,
  source::comment::Comment,
};
use serde::{Deserialize, Serialize};

#[derive(
  Clone, Queryable, Associations, Identifiable, PartialEq, Debug, Serialize, Deserialize,
)]
#[belongs_to(Comment)]
#[table_name = "person_mention"]
pub struct PersonMention {
  pub id: PersonMentionId,
  pub recipient_id: PersonId,
  pub comment_id: CommentId,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "person_mention"]
pub struct PersonMentionForm {
  pub recipient_id: PersonId,
  pub comment_id: CommentId,
  pub read: Option<bool>,
}
