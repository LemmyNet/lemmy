#[cfg(feature = "full")]
use crate::newtypes::LtreeDef;
use crate::newtypes::{CommentId, DbUrl, LanguageId, PersonId, PostId};
#[cfg(feature = "full")]
use crate::schema::{comment, comment_actions};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel::{dsl, expression_methods::NullableExpressionMethods};
#[cfg(feature = "full")]
use diesel_ltree::Ltree;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable, TS)
)]
#[cfg_attr(feature = "full", ts(export))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", diesel(table_name = comment))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A comment.
pub struct Comment {
  pub id: CommentId,
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub content: String,
  /// Whether the comment has been removed.
  pub removed: bool,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
  /// Whether the comment has been deleted by its creator.
  pub deleted: bool,
  /// The federated activity id / ap_id.
  pub ap_id: DbUrl,
  /// Whether the comment is local.
  pub local: bool,
  #[cfg(feature = "full")]
  #[cfg_attr(feature = "full", serde(with = "LtreeDef"))]
  #[cfg_attr(feature = "full", ts(type = "string"))]
  /// The path / tree location of a comment, separated by dots, ending with the comment's id. Ex:
  /// 0.24.27
  pub path: Ltree,
  #[cfg(not(feature = "full"))]
  pub path: String,
  /// Whether the comment has been distinguished(speaking officially) by a mod.
  pub distinguished: bool,
  pub language_id: LanguageId,
}

#[derive(Debug, Clone, TypedBuilder)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = comment))]
pub struct CommentInsertForm {
  #[builder(!default)]
  pub creator_id: PersonId,
  #[builder(!default)]
  pub post_id: PostId,
  #[builder(!default)]
  pub content: String,
  pub removed: Option<bool>,
  pub published: Option<DateTime<Utc>>,
  pub updated: Option<DateTime<Utc>>,
  pub deleted: Option<bool>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
  pub distinguished: Option<bool>,
  pub language_id: Option<LanguageId>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = comment))]
pub struct CommentUpdateForm {
  pub content: Option<String>,
  pub removed: Option<bool>,
  // Don't use a default naive_now here, because the create function does a lot of comment updates
  pub updated: Option<Option<DateTime<Utc>>>,
  pub deleted: Option<bool>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
  pub distinguished: Option<bool>,
  pub language_id: Option<LanguageId>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::comment::Comment)))]
#[cfg_attr(feature = "full", diesel(table_name = comment_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, comment_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct CommentLike {
  pub person_id: PersonId,
  pub comment_id: CommentId,
  #[cfg_attr(feature = "full", diesel(select_expression = comment_actions::like_score.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<comment_actions::like_score>))]
  pub score: i16,
  #[cfg_attr(feature = "full", diesel(select_expression = comment_actions::liked.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<comment_actions::liked>))]
  pub published: DateTime<Utc>,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = comment_actions))]
pub struct CommentLikeForm {
  pub person_id: PersonId,
  pub comment_id: CommentId,
  #[cfg_attr(feature = "full", diesel(column_name = like_score))]
  pub score: i16,
}

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::comment::Comment)))]
#[cfg_attr(feature = "full", diesel(table_name = comment_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, comment_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct CommentSaved {
  pub comment_id: CommentId,
  pub person_id: PersonId,
  #[cfg_attr(feature = "full", diesel(select_expression = comment_actions::saved.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<comment_actions::saved>))]
  pub published: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = comment_actions))]
pub struct CommentSavedForm {
  pub comment_id: CommentId,
  pub person_id: PersonId,
}
