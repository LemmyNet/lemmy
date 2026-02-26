use crate::newtypes::{CommunityId, LanguageId, PostId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::{PersonId, enums::PostNotificationsMode};
use lemmy_diesel_utils::dburl::DbUrl;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  i_love_jesus::CursorKeysModule,
  lemmy_db_schema_file::schema::{post, post_actions},
};

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Identifiable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = post))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = post_keys))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A post.
pub struct Post {
  pub id: PostId,
  pub name: String,
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
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  /// Whether the post is deleted.
  pub deleted: bool,
  /// Whether the post is NSFW.
  pub nsfw: bool,
  /// A title for the link.
  pub embed_title: Option<String>,
  /// A description for the link.
  pub embed_description: Option<String>,
  /// A thumbnail picture url.
  pub thumbnail_url: Option<DbUrl>,
  /// The federated activity id / ap_id.
  pub ap_id: DbUrl,
  /// Whether the post is local.
  pub local: bool,
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
  /// Time at which the post will be published. None means publish immediately.
  pub scheduled_publish_time_at: Option<DateTime<Utc>>,
  #[serde(skip)]
  /// A newest comment time, limited to 2 days, to prevent necrobumping
  pub newest_comment_time_necro_at: Option<DateTime<Utc>>,
  /// The time of the newest comment in the post, if the post has any comments.
  pub newest_comment_time_at: Option<DateTime<Utc>>,
  pub comments: i32,
  pub score: i32,
  pub upvotes: i32,
  pub downvotes: i32,
  #[serde(skip)]
  pub hot_rank: f32,
  #[serde(skip)]
  pub hot_rank_active: f32,
  #[serde(skip)]
  pub controversy_rank: f32,
  /// A rank that amplifies smaller communities
  #[serde(skip)]
  pub scaled_rank: f32,
  pub report_count: i16,
  pub unresolved_report_count: i16,
  /// If a local user posts in a remote community, the comment is hidden until it is confirmed
  /// accepted by the community (by receiving it back via federation).
  pub federation_pending: bool,
  pub embed_video_width: Option<i32>,
  pub embed_video_height: Option<i32>,
}

// TODO: FromBytes, ToBytes are only needed to develop wasm plugin, could be behind feature flag
#[derive(Debug, Clone, derive_new::new, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset,))]
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
  pub updated_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub published_at: Option<DateTime<Utc>>,
  #[new(default)]
  pub deleted: Option<bool>,
  #[new(default)]
  pub embed_title: Option<String>,
  #[new(default)]
  pub embed_description: Option<String>,
  #[new(default)]
  pub embed_video_url: Option<DbUrl>,
  #[new(default)]
  pub embed_video_width: Option<i32>,
  #[new(default)]
  pub embed_video_height: Option<i32>,
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
  pub scheduled_publish_time_at: Option<DateTime<Utc>>,
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
  pub published_at: Option<DateTime<Utc>>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
  pub deleted: Option<bool>,
  pub embed_title: Option<Option<String>>,
  pub embed_description: Option<Option<String>>,
  pub embed_video_url: Option<Option<DbUrl>>,
  pub embed_video_width: Option<Option<i32>>,
  pub embed_video_height: Option<Option<i32>>,
  pub thumbnail_url: Option<Option<DbUrl>>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
  pub language_id: Option<LanguageId>,
  pub featured_community: Option<bool>,
  pub featured_local: Option<bool>,
  pub url_content_type: Option<Option<String>>,
  pub alt_text: Option<Option<String>>,
  pub scheduled_publish_time_at: Option<Option<DateTime<Utc>>>,
  pub federation_pending: Option<bool>,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, post_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = post_actions_keys))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct PostActions {
  /// When the post was read.
  pub read_at: Option<DateTime<Utc>>,
  /// When was the last time you read the comments.
  pub read_comments_at: Option<DateTime<Utc>>,
  /// When the post was saved.
  pub saved_at: Option<DateTime<Utc>>,
  /// When the post was upvoted or downvoted.
  pub voted_at: Option<DateTime<Utc>>,
  /// When the post was hidden.
  pub hidden_at: Option<DateTime<Utc>>,
  #[serde(skip)]
  pub person_id: PersonId,
  #[serde(skip)]
  pub post_id: PostId,
  /// The number of comments you read last. Subtract this from total comments to get an unread
  /// count.
  pub read_comments_amount: Option<i32>,
  /// True if upvoted, false if downvoted. Upvote is greater than downvote.
  pub vote_is_upvote: Option<bool>,
  pub notifications: Option<PostNotificationsMode>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub struct PostLikeForm {
  pub post_id: PostId,
  pub person_id: PersonId,
  pub vote_is_upvote: Option<Option<bool>>,
  pub voted_at: Option<Option<DateTime<Utc>>>,
}

impl PostLikeForm {
  /// Pass `is_upvote: None` to remove an existing vote for this post
  pub fn new(post_id: PostId, person_id: PersonId, is_upvote: Option<bool>) -> Self {
    let voted_at = if is_upvote.is_some() {
      Some(Some(Utc::now()))
    } else {
      Some(None)
    };

    Self {
      post_id,
      person_id,
      vote_is_upvote: Some(is_upvote),
      voted_at,
    }
  }
}

#[derive(derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub struct PostSavedForm {
  pub post_id: PostId,
  pub person_id: PersonId,
  #[new(value = "Utc::now()")]
  pub saved_at: DateTime<Utc>,
}

#[derive(derive_new::new, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub(crate) struct PostReadForm {
  pub post_id: PostId,
  pub person_id: PersonId,
  #[new(value = "Utc::now()")]
  pub read_at: DateTime<Utc>,
}

#[derive(derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub struct PostReadCommentsForm {
  pub post_id: PostId,
  pub person_id: PersonId,
  pub read_comments_amount: i32,
  #[new(value = "Utc::now()")]
  pub read_comments_at: DateTime<Utc>,
}

#[derive(derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub struct PostHideForm {
  pub post_id: PostId,
  pub person_id: PersonId,
  #[new(value = "Utc::now()")]
  pub hidden_at: DateTime<Utc>,
}
