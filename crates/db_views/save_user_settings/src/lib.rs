use lemmy_db_schema::{newtypes::LanguageId, sensitive::SensitiveString};
use lemmy_db_schema_file::enums::{
  CommentSortType, ListingType, PostListingMode, PostSortType, VoteShow,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Saves settings for your user.
pub struct SaveUserSettings {
  /// Show nsfw posts.
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_nsfw: Option<bool>,
  /// Blur nsfw posts.
  #[cfg_attr(feature = "full", ts(optional))]
  pub blur_nsfw: Option<bool>,
  /// Your user's theme.
  #[cfg_attr(feature = "full", ts(optional))]
  pub theme: Option<String>,
  /// The default post listing type, usually "local"
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_listing_type: Option<ListingType>,
  /// A post-view mode that changes how multiple post listings look.
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_listing_mode: Option<PostListingMode>,
  /// The default post sort, usually "active"
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_sort_type: Option<PostSortType>,
  /// A default time range limit to apply to post sorts, in seconds. 0 means none.
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_time_range_seconds: Option<i32>,
  /// The default comment sort, usually "hot"
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_comment_sort_type: Option<CommentSortType>,
  /// The language of the lemmy interface
  #[cfg_attr(feature = "full", ts(optional))]
  pub interface_language: Option<String>,
  /// Your display name, which can contain strange characters, and does not need to be unique.
  #[cfg_attr(feature = "full", ts(optional))]
  pub display_name: Option<String>,
  /// Your email.
  #[cfg_attr(feature = "full", ts(optional))]
  pub email: Option<SensitiveString>,
  /// Your bio / info, in markdown.
  #[cfg_attr(feature = "full", ts(optional))]
  pub bio: Option<String>,
  /// Your matrix user id. Ex: @my_user:matrix.org
  #[cfg_attr(feature = "full", ts(optional))]
  pub matrix_user_id: Option<String>,
  /// Whether to show or hide avatars.
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_avatars: Option<bool>,
  /// Sends notifications to your email.
  #[cfg_attr(feature = "full", ts(optional))]
  pub send_notifications_to_email: Option<bool>,
  /// Whether this account is a bot account. Users can hide these accounts easily if they wish.
  #[cfg_attr(feature = "full", ts(optional))]
  pub bot_account: Option<bool>,
  /// Whether to show bot accounts.
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_bot_accounts: Option<bool>,
  /// Whether to show read posts.
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_read_posts: Option<bool>,
  /// A list of languages you are able to see discussion in.
  #[cfg_attr(feature = "full", ts(optional))]
  pub discussion_languages: Option<Vec<LanguageId>>,
  // A list of keywords used for blocking posts having them in title,url or body.
  #[cfg_attr(feature = "full", ts(optional))]
  pub blocking_keywords: Option<Vec<String>>,
  /// Open links in a new tab
  #[cfg_attr(feature = "full", ts(optional))]
  pub open_links_in_new_tab: Option<bool>,
  /// Enable infinite scroll
  #[cfg_attr(feature = "full", ts(optional))]
  pub infinite_scroll_enabled: Option<bool>,
  /// Whether to allow keyboard navigation (for browsing and interacting with posts and comments).
  #[cfg_attr(feature = "full", ts(optional))]
  pub enable_keyboard_navigation: Option<bool>,
  /// Whether user avatars or inline images in the UI that are gifs should be allowed to play or
  /// should be paused
  #[cfg_attr(feature = "full", ts(optional))]
  pub enable_animated_images: Option<bool>,
  /// Whether a user can send / receive private messages
  #[cfg_attr(feature = "full", ts(optional))]
  pub enable_private_messages: Option<bool>,
  /// Whether to auto-collapse bot comments.
  #[cfg_attr(feature = "full", ts(optional))]
  pub collapse_bot_comments: Option<bool>,
  /// Some vote display mode settings
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_scores: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_upvotes: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_downvotes: Option<VoteShow>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_upvote_percentage: Option<bool>,
  /// Whether to automatically mark fetched posts as read.
  #[cfg_attr(feature = "full", ts(optional))]
  pub auto_mark_fetched_posts_as_read: Option<bool>,
  /// Whether to hide posts containing images/videos.
  #[cfg_attr(feature = "full", ts(optional))]
  pub hide_media: Option<bool>,
}
