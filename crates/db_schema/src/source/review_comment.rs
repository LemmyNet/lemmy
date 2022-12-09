use crate::newtypes::{CommentId, PersonId, ReviewCommentId};
#[cfg(feature = "full")]
use crate::schema::review_comment;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::comment::Comment)))]
#[cfg_attr(feature = "full", diesel(table_name = review_comment))]
pub struct ReviewComment {
  pub id: ReviewCommentId,
  pub comment_id: CommentId,
  pub approved: bool,
  pub approver_id: Option<PersonId>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = review_comment))]
pub struct ReviewCommentForm {
  pub comment_id: CommentId,
}
