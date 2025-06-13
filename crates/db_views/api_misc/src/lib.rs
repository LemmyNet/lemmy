use lemmy_db_schema::{
  newtypes::{InstanceId, LanguageId, PaginationCursor, PersonId},
  sensitive::SensitiveString,
  source::{community::Community, instance::Instance, login_token::LoginToken, person::Person},
};
use lemmy_db_schema_file::enums::{
  CommentSortType,
  ListingType,
  PostListingMode,
  PostSortType,
  VoteShow,
};
use lemmy_db_views_community_follower::CommunityFollowerView;
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::PostView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;
#[cfg(feature = "full")]
use {
  extism::FromBytes,
  extism_convert::{encoding, Json},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A captcha response.
pub struct CaptchaResponse {
  /// A Base64 encoded png
  pub png: String,
  /// A Base64 encoded wav audio
  pub wav: String,
  /// The UUID for the captcha item.
  pub uuid: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Changes your account password.
pub struct ChangePassword {
  pub new_password: SensitiveString,
  pub new_password_verify: SensitiveString,
  pub old_password: SensitiveString,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a tagline
pub struct CreateTagline {
  pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Delete your account.
pub struct DeleteAccount {
  pub password: SensitiveString,
  pub delete_content: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A wrapper for the captcha response.
pub struct GetCaptchaResponse {
  /// Will be None if captchas are disabled.
  pub ok: Option<CaptchaResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct GenerateTotpSecretResponse {
  pub totp_secret_url: SensitiveString,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ListLoginsResponse {
  pub logins: Vec<LoginToken>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets your hidden posts.
pub struct ListPersonHidden {
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// You hidden posts response.
pub struct ListPersonHiddenResponse {
  pub hidden: Vec<PostView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Gets your read posts.
pub struct ListPersonRead {
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// You read posts response.
pub struct ListPersonReadResponse {
  pub read: Vec<PostView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Logging into lemmy.
///
/// Note: Banned users can still log in, to be able to do certain things like delete
/// their account.
pub struct Login {
  pub username_or_email: SensitiveString,
  pub password: SensitiveString,
  /// May be required, if totp is enabled for their account.
  pub totp_2fa_token: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A response for your login.
pub struct LoginResponse {
  /// This is None in response to `Register` if email verification is enabled, or the server
  /// requires registration applications.
  pub jwt: Option<SensitiveString>,
  /// If registration applications are required, this will return true for a signup response.
  pub registration_created: bool,
  /// If email verifications are required, this will return true for a signup response.
  pub verify_email_sent: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Make a note for a person.
///
/// An empty string deletes the note.
pub struct NotePerson {
  pub person_id: PersonId,
  pub note: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Your user info.
pub struct MyUserInfo {
  pub local_user_view: LocalUserView,
  pub follows: Vec<CommunityFollowerView>,
  pub moderates: Vec<CommunityModeratorView>,
  pub community_blocks: Vec<Community>,
  pub instance_blocks: Vec<Instance>,
  pub person_blocks: Vec<Person>,
  pub keyword_blocks: Vec<String>,
  pub discussion_languages: Vec<LanguageId>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Change your password after receiving a reset request.
pub struct PasswordChangeAfterReset {
  pub token: SensitiveString,
  pub password: SensitiveString,
  pub password_verify: SensitiveString,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Reset your password via email.
pub struct PasswordReset {
  pub email: SensitiveString,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(FromBytes))]
#[cfg_attr(feature = "full", encoding(Json))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct PluginMetadata {
  name: String,
  url: Url,
  description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Make a request to resend your verification email.
pub struct ResendVerificationEmail {
  pub email: SensitiveString,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Does an apub fetch for an object.
pub struct ResolveObject {
  /// Can be the full url, or a shortened version like: !fediverse@lemmy.ml
  pub q: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Saves settings for your user.
pub struct SaveUserSettings {
  /// Show nsfw posts.
  pub show_nsfw: Option<bool>,
  /// Blur nsfw posts.
  pub blur_nsfw: Option<bool>,
  /// Your user's theme.
  pub theme: Option<String>,
  /// The default post listing type, usually "local"
  pub default_listing_type: Option<ListingType>,
  /// A post-view mode that changes how multiple post listings look.
  pub post_listing_mode: Option<PostListingMode>,
  /// The default post sort, usually "active"
  pub default_post_sort_type: Option<PostSortType>,
  /// A default time range limit to apply to post sorts, in seconds. 0 means none.
  pub default_post_time_range_seconds: Option<i32>,
  /// The default comment sort, usually "hot"
  pub default_comment_sort_type: Option<CommentSortType>,
  /// The language of the lemmy interface
  pub interface_language: Option<String>,
  /// Your display name, which can contain strange characters, and does not need to be unique.
  pub display_name: Option<String>,
  /// Your email.
  pub email: Option<SensitiveString>,
  /// Your bio / info, in markdown.
  pub bio: Option<String>,
  /// Your matrix user id. Ex: @my_user:matrix.org
  pub matrix_user_id: Option<String>,
  /// Whether to show or hide avatars.
  pub show_avatars: Option<bool>,
  /// Sends notifications to your email.
  pub send_notifications_to_email: Option<bool>,
  /// Whether this account is a bot account. Users can hide these accounts easily if they wish.
  pub bot_account: Option<bool>,
  /// Whether to show bot accounts.
  pub show_bot_accounts: Option<bool>,
  /// Whether to show read posts.
  pub show_read_posts: Option<bool>,
  /// A list of languages you are able to see discussion in.
  pub discussion_languages: Option<Vec<LanguageId>>,
  // A list of keywords used for blocking posts having them in title,url or body.
  pub blocking_keywords: Option<Vec<String>>,
  /// Open links in a new tab
  pub open_links_in_new_tab: Option<bool>,
  /// Enable infinite scroll
  pub infinite_scroll_enabled: Option<bool>,
  /// Whether to allow keyboard navigation (for browsing and interacting with posts and comments).
  pub enable_keyboard_navigation: Option<bool>,
  /// Whether user avatars or inline images in the UI that are gifs should be allowed to play or
  /// should be paused
  pub enable_animated_images: Option<bool>,
  /// Whether a user can send / receive private messages
  pub enable_private_messages: Option<bool>,
  /// Whether to auto-collapse bot comments.
  pub collapse_bot_comments: Option<bool>,
  /// Some vote display mode settings
  pub show_scores: Option<bool>,
  pub show_upvotes: Option<bool>,
  pub show_downvotes: Option<VoteShow>,
  pub show_upvote_percentage: Option<bool>,
  /// Whether to automatically mark fetched posts as read.
  pub auto_mark_fetched_posts_as_read: Option<bool>,
  /// Whether to hide posts containing images/videos.
  pub hide_media: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A response that completes successfully.
pub struct SuccessResponse {
  pub success: bool,
}

impl Default for SuccessResponse {
  fn default() -> Self {
    SuccessResponse { success: true }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct UpdateTotp {
  pub totp_token: String,
  pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct UpdateTotpResponse {
  pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Block an instance as user
pub struct UserBlockInstanceParams {
  pub instance_id: InstanceId,
  pub block: bool,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Verify your email.
pub struct VerifyEmail {
  pub token: String,
}
