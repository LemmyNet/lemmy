use crate::newtypes::{CommunityId, DbUrl, LanguageId, PersonId, PostId};
#[cfg(feature = "full")]
use crate::schema::{post, post_actions};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel::{dsl, expression_methods::NullableExpressionMethods};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = post))]
#[cfg_attr(feature = "full", ts(export))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A post.
pub struct Post {
  pub id: PostId,
  pub name: String,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  /// An optional link / url for the post.
  pub url: Option<DbUrl>,
  /// An optional post body, in markdown.
  pub body: Option<String>,
  pub creator_id: PersonId,
  pub community_id: CommunityId,
  /// Whether the post is removed.
  pub removed: bool,
  /// Whether the post is locked.
  pub locked: bool,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
  /// Whether the post is deleted.
  pub deleted: bool,
  /// Whether the post is NSFW.
  pub nsfw: bool,
  /// A title for the link.
  pub embed_title: Option<String>,
  /// A description for the link.
  pub embed_description: Option<String>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  /// A thumbnail picture url.
  pub thumbnail_url: Option<DbUrl>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  /// The federated activity id / ap_id.
  pub ap_id: DbUrl,
  /// Whether the post is local.
  pub local: bool,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  /// A video url for the link.
  pub embed_video_url: Option<DbUrl>,
  pub language_id: LanguageId,
  /// Whether the post is featured to its community.
  pub featured_community: bool,
  /// Whether the post is featured to its site.
  pub featured_local: bool,
  pub url_content_type: Option<String>,
  /// An optional alt_text, usable for image posts.
  pub alt_text: Option<String>,
}

#[derive(Debug, Clone, TypedBuilder)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post))]
pub struct PostInsertForm {
  #[builder(!default)]
  pub name: String,
  #[builder(!default)]
  pub creator_id: PersonId,
  #[builder(!default)]
  pub community_id: CommunityId,
  pub nsfw: Option<bool>,
  pub url: Option<DbUrl>,
  pub body: Option<String>,
  pub removed: Option<bool>,
  pub locked: Option<bool>,
  pub updated: Option<DateTime<Utc>>,
  pub published: Option<DateTime<Utc>>,
  pub deleted: Option<bool>,
  pub embed_title: Option<String>,
  pub embed_description: Option<String>,
  pub embed_video_url: Option<DbUrl>,
  pub thumbnail_url: Option<DbUrl>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
  pub language_id: Option<LanguageId>,
  pub featured_community: Option<bool>,
  pub featured_local: Option<bool>,
  pub url_content_type: Option<String>,
  pub alt_text: Option<String>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post))]
pub struct PostUpdateForm {
  pub name: Option<String>,
  pub nsfw: Option<bool>,
  pub url: Option<Option<DbUrl>>,
  pub body: Option<Option<String>>,
  pub removed: Option<bool>,
  pub locked: Option<bool>,
  pub published: Option<DateTime<Utc>>,
  pub updated: Option<Option<DateTime<Utc>>>,
  pub deleted: Option<bool>,
  pub embed_title: Option<Option<String>>,
  pub embed_description: Option<Option<String>>,
  pub embed_video_url: Option<Option<DbUrl>>,
  pub thumbnail_url: Option<Option<DbUrl>>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
  pub language_id: Option<LanguageId>,
  pub featured_community: Option<bool>,
  pub featured_local: Option<bool>,
  pub url_content_type: Option<Option<String>>,
  pub alt_text: Option<Option<String>>,
}

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, post_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct PostLike {
  pub post_id: PostId,
  pub person_id: PersonId,
  #[diesel(select_expression = post_actions::like_score.assume_not_null())]
  #[diesel(select_expression_type = dsl::AssumeNotNull<post_actions::like_score>)]
  pub score: i16,
  #[diesel(select_expression = post_actions::liked.assume_not_null())]
  #[diesel(select_expression_type = dsl::AssumeNotNull<post_actions::liked>)]
  pub published: DateTime<Utc>,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub struct PostLikeForm {
  pub post_id: PostId,
  pub person_id: PersonId,
  #[cfg_attr(feature = "full", diesel(column_name = like_score))]
  pub score: i16,
}

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, post_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct PostSaved {
  pub post_id: PostId,
  pub person_id: PersonId,
  #[diesel(select_expression = post_actions::saved.assume_not_null())]
  #[diesel(select_expression_type = dsl::AssumeNotNull<post_actions::saved>)]
  pub published: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub struct PostSavedForm {
  pub post_id: PostId,
  pub person_id: PersonId,
}

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, post_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct PostRead {
  pub post_id: PostId,
  pub person_id: PersonId,
  #[diesel(select_expression = post_actions::read.assume_not_null())]
  #[diesel(select_expression_type = dsl::AssumeNotNull<post_actions::read>)]
  pub published: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub(crate) struct PostReadForm {
  pub post_id: PostId,
  pub person_id: PersonId,
}

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, post_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct PostHide {
  pub post_id: PostId,
  pub person_id: PersonId,
  #[diesel(select_expression = post_actions::hidden.assume_not_null())]
  #[diesel(select_expression_type = dsl::AssumeNotNull<post_actions::hidden>)]
  pub published: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub(crate) struct PostHideForm {
  pub post_id: PostId,
  pub person_id: PersonId,
}
