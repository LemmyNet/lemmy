use crate::newtypes::{CommunityId, DbUrl, LanguageId, PersonId, PostId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  i_love_jesus::CursorKeysModule,
  lemmy_db_schema_file::schema::{post, post_actions},
  ts_rs::TS,
};

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable, TS, CursorKeysModule)
)]
#[cfg_attr(feature = "full", ts(export))]
#[cfg_attr(feature = "full", diesel(table_name = post))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = post_keys))]
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
  pub comments: i64,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  #[serde(skip)]
  /// A newest comment time, limited to 2 days, to prevent necrobumping
  pub newest_comment_time_necro: DateTime<Utc>,
  /// The time of the newest comment in the post.
  pub newest_comment_time: DateTime<Utc>,
  #[serde(skip)]
  pub hot_rank: f64,
  #[serde(skip)]
  pub hot_rank_active: f64,
  #[serde(skip)]
  pub controversy_rank: f64,
  /// A rank that amplifies smaller communities
  #[serde(skip)]
  pub scaled_rank: f64,
  pub report_count: i16,
  pub unresolved_report_count: i16,
  /// If a local user posts in a remote community, the comment is hidden until it is confirmed
  /// accepted by the community (by receiving it back via federation).
  pub federation_pending: bool,
}

// TODO: FromBytes, ToBytes are only needed to develop wasm plugin, could be behind feature flag
#[derive(Debug, Clone, derive_new::new)]
#[cfg_attr(
  feature = "full",
  derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
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
  #[new(default)]
  pub federation_pending: Option<bool>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset, Serialize, Deserialize))]
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
  pub federation_pending: Option<bool>,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations, TS,)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, post_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PostActions {
  pub post_id: PostId,
  pub person_id: PersonId,
  #[cfg_attr(feature = "full", ts(optional))]
  /// When the post was read.
  pub read: Option<DateTime<Utc>>,
  #[cfg_attr(feature = "full", ts(optional))]
  /// When was the last time you read the comments.
  pub read_comments: Option<DateTime<Utc>>,
  #[cfg_attr(feature = "full", ts(optional))]
  /// The number of comments you read last. Subtract this from total comments to get an unread
  /// count.
  pub read_comments_amount: Option<i64>,
  #[cfg_attr(feature = "full", ts(optional))]
  /// When the post was saved.
  pub saved: Option<DateTime<Utc>>,
  #[cfg_attr(feature = "full", ts(optional))]
  /// When the post was liked.
  pub liked: Option<DateTime<Utc>>,
  #[cfg_attr(feature = "full", ts(optional))]
  /// The like / score of the post.
  pub like_score: Option<i16>,
  #[cfg_attr(feature = "full", ts(optional))]
  /// When the post was hidden.
  pub hidden: Option<DateTime<Utc>>,
  // TODO: use select_expression with coalesce to change this to bool (cant get it to compile)
  pub subscribed: Option<bool>,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(
  feature = "full",
  derive(Insertable, AsChangeset, Serialize, Deserialize)
)]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub struct PostLikeForm {
  pub post_id: PostId,
  pub person_id: PersonId,
  pub like_score: i16,
  #[new(value = "Utc::now()")]
  pub liked: DateTime<Utc>,
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

#[derive(derive_new::new, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub struct PostReadForm {
  pub post_id: PostId,
  pub person_id: PersonId,
  #[new(value = "Utc::now()")]
  pub read: DateTime<Utc>,
}

#[derive(derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub struct PostReadCommentsForm {
  pub post_id: PostId,
  pub person_id: PersonId,
  pub read_comments_amount: i64,
  #[new(value = "Utc::now()")]
  pub read_comments: DateTime<Utc>,
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

#[derive(derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub struct PostSubscribeForm {
  pub post_id: PostId,
  pub person_id: PersonId,
}

#[derive(PartialEq, Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, CursorKeysModule))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = post_actions_keys))]
/// Sorted timestamps of actions on a post.
pub struct PostActionsCursor {
  pub read: Option<DateTime<Utc>>,
}
