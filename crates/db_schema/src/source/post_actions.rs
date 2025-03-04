use crate::newtypes::{PersonId, PostId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {
  crate::schema::post_actions,
  diesel::{dsl, expression_methods::NullableExpressionMethods},
};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, post_id)))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::person::Person)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// Aggregate data for a person's post.
pub struct PostActions {
  pub person_id: PersonId,
  pub post_id: PostId,
  /// The number of comments they've read on that post.
  ///
  /// This is updated to the current post comment count every time they view a post.
  #[cfg_attr(feature = "full", diesel(select_expression = post_actions::read_comments_amount.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<post_actions::read_comments_amount>))]
  pub read_comments: i64,
  #[cfg_attr(feature = "full", diesel(select_expression = post_actions::read_comments.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<post_actions::read_comments>))]
  pub published: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub struct PostActionsForm {
  pub person_id: PersonId,
  pub post_id: PostId,
  #[cfg_attr(feature = "full", diesel(column_name = read_comments_amount))]
  pub read_comments: i64,
}
