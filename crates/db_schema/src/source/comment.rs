use crate::newtypes::{CommentId, DbUrl, LanguageId, PersonId, PostId};
#[cfg(feature = "full")]
use crate::utils::{
  functions::get_score,
  queryable::{ChangeNullTo, LikeScore},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  crate::newtypes::LtreeDef,
  diesel_ltree::Ltree,
  i_love_jesus::CursorKeysModule,
  lemmy_db_schema_file::schema::{comment, comment_actions},
};

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", diesel(table_name = comment))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A comment.
pub struct Comment {
  pub id: CommentId,
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub content: String,
  /// Whether the comment has been removed.
  pub removed: bool,
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  /// Whether the comment has been deleted by its creator.
  pub deleted: bool,
  /// The federated activity id / ap_id.
  pub ap_id: DbUrl,
  /// Whether the comment is local.
  pub local: bool,
  #[cfg(feature = "full")]
  #[cfg_attr(feature = "full", serde(with = "LtreeDef"))]
  #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
  /// The path / tree location of a comment, separated by dots, ending with the comment's id. Ex:
  /// 0.24.27
  pub path: Ltree,
  #[cfg(not(feature = "full"))]
  pub path: String,
  /// Whether the comment has been distinguished(speaking officially) by a mod.
  pub distinguished: bool,
  pub language_id: LanguageId,
  #[diesel(select_expression = get_score(comment::non_1_upvotes, comment::non_0_downvotes))]
  pub score: i32,
  #[cfg_attr(feature = "full", diesel(deserialize_as = ChangeNullTo<1, i32>))]
  #[cfg_attr(feature = "full", diesel(column_name = non_1_upvotes))]
  pub upvotes: i32,
  #[cfg_attr(feature = "full", diesel(deserialize_as = ChangeNullTo<0, i32>))]
  #[cfg_attr(feature = "full", diesel(column_name = non_0_downvotes))]
  pub downvotes: i32,
  /// The total number of children in this comment branch.
  #[cfg_attr(feature = "full", diesel(deserialize_as = ChangeNullTo<0, i32>))]
  #[cfg_attr(feature = "full", diesel(column_name = non_0_child_count))]
  pub child_count: i32,
  #[serde(skip)]
  pub age: Option<i16>,
  #[cfg_attr(feature = "full", diesel(deserialize_as = ChangeNullTo<0, i16>))]
  #[cfg_attr(feature = "full", diesel(column_name = non_0_report_count))]
  pub report_count: i16,
  #[cfg_attr(feature = "full", diesel(deserialize_as = ChangeNullTo<0, i16>))]
  #[cfg_attr(feature = "full", diesel(column_name = non_0_unresolved_report_count))]
  pub unresolved_report_count: i16,
  /// If a local user comments in a remote community, the comment is hidden until it is confirmed
  /// accepted by the community (by receiving it back via federation).
  pub federation_pending: bool,
}

#[cfg(feature = "full")]
#[derive(Queryable, Selectable, CursorKeysModule)]
#[diesel(table_name = comment)]
#[cursor_keys_module(name = comment_keys)]
pub struct CommentCursorData {
  pub id: CommentId,
  pub published_at: DateTime<Utc>,
  pub path: Ltree,
  pub distinguished: bool,
  pub non_1_upvotes: Option<i32>,
  pub non_0_downvotes: Option<i32>,
  pub age: Option<i16>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(
  feature = "full",
  derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = comment))]
pub struct CommentInsertForm {
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub content: String,
  #[new(default)]
  pub removed: Option<bool>,
  #[new(default)]
  pub published_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub updated_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub deleted: Option<bool>,
  #[new(default)]
  pub ap_id: Option<DbUrl>,
  #[new(default)]
  pub local: Option<bool>,
  #[new(default)]
  pub distinguished: Option<bool>,
  #[new(default)]
  pub language_id: Option<LanguageId>,
  #[new(default)]
  pub federation_pending: Option<bool>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset, Serialize, Deserialize))]
#[cfg_attr(feature = "full", diesel(table_name = comment))]
pub struct CommentUpdateForm {
  pub content: Option<String>,
  pub removed: Option<bool>,
  // Don't use a default Utc::now here, because the create function does a lot of comment updates
  pub updated_at: Option<Option<DateTime<Utc>>>,
  pub deleted: Option<bool>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
  pub distinguished: Option<bool>,
  pub language_id: Option<LanguageId>,
  pub federation_pending: Option<bool>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::comment::Comment)))]
#[cfg_attr(feature = "full", diesel(table_name = comment_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, comment_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct CommentActions {
  #[serde(skip)]
  pub person_id: PersonId,
  #[serde(skip)]
  pub comment_id: CommentId,
  /// The like / score for the comment.
  #[cfg_attr(feature = "full", diesel(deserialize_as = LikeScore))]
  #[cfg_attr(feature = "full", diesel(column_name = like_score_is_positive))]
  pub like_score: Option<i16>,
  /// When the comment was liked.
  pub liked_at: Option<DateTime<Utc>>,
  /// When the comment was saved.
  pub saved_at: Option<DateTime<Utc>>,
}

#[cfg(feature = "full")]
#[derive(Queryable, Selectable, CursorKeysModule)]
#[diesel(table_name = comment_actions)]
#[cursor_keys_module(name = comment_actions_keys)]
pub struct CommentActionsCursorData {
  pub comment_id: PostId,
  pub liked_at: Option<DateTime<Utc>>,
  /// Upvote is greater than downvote.
  pub like_score_is_positive: Option<bool>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Serialize, Deserialize))]
pub struct CommentLikeForm {
  pub person_id: PersonId,
  pub comment_id: CommentId,
  pub like_score: i16,
  #[new(value = "Utc::now()")]
  pub liked_at: DateTime<Utc>,
}

#[derive(derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = comment_actions))]
pub struct CommentSavedForm {
  pub person_id: PersonId,
  pub comment_id: CommentId,
  #[new(value = "Utc::now()")]
  pub saved_at: DateTime<Utc>,
}
