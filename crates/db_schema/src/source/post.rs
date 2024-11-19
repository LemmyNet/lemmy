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
  /// An optional link / url for the post.
  #[cfg_attr(feature = "full", ts(optional))]
  pub url: Option<DbUrl>,
  /// An optional post body, in markdown.
  #[cfg_attr(feature = "full", ts(optional))]
  pub body: Option<String>,
  pub creator_id: PersonId,
  pub community_id: CommunityId,
  /// Whether the post is removed.
  pub removed: bool,
  /// Whether the post is locked.
  pub locked: bool,
  pub published: DateTime<Utc>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub updated: Option<DateTime<Utc>>,
  /// Whether the post is deleted.
  pub deleted: bool,
  /// Whether the post is NSFW.
  pub nsfw: bool,
  /// A title for the link.
  #[cfg_attr(feature = "full", ts(optional))]
  pub embed_title: Option<String>,
  /// A description for the link.
  #[cfg_attr(feature = "full", ts(optional))]
  pub embed_description: Option<String>,
  /// A thumbnail picture url.
  #[cfg_attr(feature = "full", ts(optional))]
  pub thumbnail_url: Option<DbUrl>,
  /// The federated activity id / ap_id.
  pub ap_id: DbUrl,
  /// Whether the post is local.
  pub local: bool,
  /// A video url for the link.
  #[cfg_attr(feature = "full", ts(optional))]
  pub embed_video_url: Option<DbUrl>,
  pub language_id: LanguageId,
  /// Whether the post is featured to its community.
  pub featured_community: bool,
  /// Whether the post is featured to its site.
  pub featured_local: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub url_content_type: Option<String>,
  /// An optional alt_text, usable for image posts.
  #[cfg_attr(feature = "full", ts(optional))]
  pub alt_text: Option<String>,
  /// Time at which the post will be published. None means publish immediately.
  #[cfg_attr(feature = "full", ts(optional))]
  pub scheduled_publish_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post))]
pub struct PostInsertForm {
  pub name: String,
  pub creator_id: PersonId,
  pub community_id: CommunityId,
  #[new(default)]
  pub nsfw: Option<bool>,
  #[new(default)]
  pub url: Option<DbUrl>,
  #[new(default)]
  pub body: Option<String>,
  #[new(default)]
  pub removed: Option<bool>,
  #[new(default)]
  pub locked: Option<bool>,
  #[new(default)]
  pub updated: Option<DateTime<Utc>>,
  #[new(default)]
  pub published: Option<DateTime<Utc>>,
  #[new(default)]
  pub deleted: Option<bool>,
  #[new(default)]
  pub embed_title: Option<String>,
  #[new(default)]
  pub embed_description: Option<String>,
  #[new(default)]
  pub embed_video_url: Option<DbUrl>,
  #[new(default)]
  pub thumbnail_url: Option<DbUrl>,
  #[new(default)]
  pub ap_id: Option<DbUrl>,
  #[new(default)]
  pub local: Option<bool>,
  #[new(default)]
  pub language_id: Option<LanguageId>,
  #[new(default)]
  pub featured_community: Option<bool>,
  #[new(default)]
  pub featured_local: Option<bool>,
  #[new(default)]
  pub url_content_type: Option<String>,
  #[new(default)]
  pub alt_text: Option<String>,
  #[new(default)]
  pub scheduled_publish_time: Option<DateTime<Utc>>,
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
  pub scheduled_publish_time: Option<Option<DateTime<Utc>>>,
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
  #[cfg_attr(feature = "full", diesel(select_expression = post_actions::like_score.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<post_actions::like_score>))]
  pub score: i16,
  #[cfg_attr(feature = "full", diesel(select_expression = post_actions::liked.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<post_actions::liked>))]
  pub published: DateTime<Utc>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub struct PostLikeForm {
  pub post_id: PostId,
  pub person_id: PersonId,
  #[cfg_attr(feature = "full", diesel(column_name = like_score))]
  pub score: i16,
  #[new(value = "Utc::now()")]
  pub liked: DateTime<Utc>,
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
  #[cfg_attr(feature = "full", diesel(select_expression = post_actions::saved.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<post_actions::saved>))]
  pub published: DateTime<Utc>,
}

#[derive(derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub struct PostSavedForm {
  pub post_id: PostId,
  pub person_id: PersonId,
  #[new(value = "Utc::now()")]
  pub saved: DateTime<Utc>,
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
  #[cfg_attr(feature = "full", diesel(select_expression = post_actions::read.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<post_actions::read>))]
  pub published: DateTime<Utc>,
}

#[derive(derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub struct PostReadForm {
  pub post_id: PostId,
  pub person_id: PersonId,
  #[new(value = "Utc::now()")]
  pub read: DateTime<Utc>,
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
  #[cfg_attr(feature = "full", diesel(select_expression = post_actions::hidden.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<post_actions::hidden>))]
  pub published: DateTime<Utc>,
}

#[derive(derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub struct PostHideForm {
  pub post_id: PostId,
  pub person_id: PersonId,
  #[new(value = "Utc::now()")]
  pub hidden: DateTime<Utc>,
}
