use lemmy_db_schema::{
  newtypes::{
    CommentReplyId,
    CommunityId,
    LanguageId,
    PersonCommentMentionId,
    PersonId,
    PersonPostMentionId,
  },
  sensitive::SensitiveString,
  source::{login_token::LoginToken, site::Site},
  CommentSortType,
  InboxDataType,
  ListingType,
  PersonContentType,
  PostListingMode,
  PostSortType,
};
use lemmy_db_views::structs::{
  CommunityModeratorView,
  InboxCombinedPaginationCursor,
  InboxCombinedView,
  LocalImageView,
  PersonContentCombinedPaginationCursor,
  PersonContentCombinedView,
  PersonSavedCombinedPaginationCursor,
  PersonView,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Logging into lemmy.
pub struct Login {
  pub username_or_email: SensitiveString,
  pub password: SensitiveString,
  /// May be required, if totp is enabled for their account.
  #[cfg_attr(feature = "full", ts(optional))]
  pub totp_2fa_token: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Register / Sign up to lemmy.
pub struct Register {
  pub username: String,
  pub password: SensitiveString,
  pub password_verify: SensitiveString,
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_nsfw: Option<bool>,
  /// email is mandatory if email verification is enabled on the server
  #[cfg_attr(feature = "full", ts(optional))]
  pub email: Option<SensitiveString>,
  /// The UUID of the captcha item.
  #[cfg_attr(feature = "full", ts(optional))]
  pub captcha_uuid: Option<String>,
  /// Your captcha answer.
  #[cfg_attr(feature = "full", ts(optional))]
  pub captcha_answer: Option<String>,
  /// A form field to trick signup bots. Should be None.
  #[cfg_attr(feature = "full", ts(optional))]
  pub honeypot: Option<String>,
  /// An answer is mandatory if require application is enabled on the server
  #[cfg_attr(feature = "full", ts(optional))]
  pub answer: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A wrapper for the captcha response.
pub struct GetCaptchaResponse {
  /// Will be None if captchas are disabled.
  #[cfg_attr(feature = "full", ts(optional))]
  pub ok: Option<CaptchaResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A captcha response.
pub struct CaptchaResponse {
  /// A Base64 encoded png
  pub png: String,
  /// A Base64 encoded wav audio
  pub wav: String,
  /// The UUID for the captcha item.
  pub uuid: String,
}

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
  pub show_downvotes: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_upvote_percentage: Option<bool>,
  /// Whether to automatically mark fetched posts as read.
  #[cfg_attr(feature = "full", ts(optional))]
  pub auto_mark_fetched_posts_as_read: Option<bool>,
  /// Whether to hide posts containing images/videos.
  #[cfg_attr(feature = "full", ts(optional))]
  pub hide_media: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Changes your account password.
pub struct ChangePassword {
  pub new_password: SensitiveString,
  pub new_password_verify: SensitiveString,
  pub old_password: SensitiveString,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response for your login.
pub struct LoginResponse {
  /// This is None in response to `Register` if email verification is enabled, or the server
  /// requires registration applications.
  #[cfg_attr(feature = "full", ts(optional))]
  pub jwt: Option<SensitiveString>,
  /// If registration applications are required, this will return true for a signup response.
  pub registration_created: bool,
  /// If email verifications are required, this will return true for a signup response.
  pub verify_email_sent: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Gets a person's details.
///
/// Either person_id, or username are required.
pub struct GetPersonDetails {
  #[cfg_attr(feature = "full", ts(optional))]
  pub person_id: Option<PersonId>,
  /// Example: dessalines , or dessalines@xyz.tld
  #[cfg_attr(feature = "full", ts(optional))]
  pub username: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A person's details response.
pub struct GetPersonDetailsResponse {
  pub person_view: PersonView,
  #[cfg_attr(feature = "full", ts(optional))]
  pub site: Option<Site>,
  pub moderates: Vec<CommunityModeratorView>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Gets a person's content (posts and comments)
///
/// Either person_id, or username are required.
pub struct ListPersonContent {
  #[cfg_attr(feature = "full", ts(optional))]
  pub type_: Option<PersonContentType>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub person_id: Option<PersonId>,
  /// Example: dessalines , or dessalines@xyz.tld
  #[cfg_attr(feature = "full", ts(optional))]
  pub username: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_cursor: Option<PersonContentCombinedPaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_back: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A person's content response.
pub struct ListPersonContentResponse {
  pub content: Vec<PersonContentCombinedView>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Gets your saved posts and comments
pub struct ListPersonSaved {
  #[cfg_attr(feature = "full", ts(optional))]
  pub type_: Option<PersonContentType>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_cursor: Option<PersonSavedCombinedPaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_back: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A person's saved content response.
pub struct ListPersonSavedResponse {
  pub saved: Vec<PersonContentCombinedView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Adds an admin to a site.
pub struct AddAdmin {
  pub person_id: PersonId,
  pub added: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response of current admins.
pub struct AddAdminResponse {
  pub admins: Vec<PersonView>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Ban a person from the site.
pub struct BanPerson {
  pub person_id: PersonId,
  pub ban: bool,
  /// Optionally remove or restore all their data. Useful for new troll accounts.
  /// If ban is true, then this means remove. If ban is false, it means restore.
  #[cfg_attr(feature = "full", ts(optional))]
  pub remove_or_restore_data: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
  /// A time that the ban will expire, in unix epoch seconds.
  ///
  /// An i64 unix timestamp is used for a simpler API client implementation.
  #[cfg_attr(feature = "full", ts(optional))]
  pub expires: Option<i64>,
}

// TODO, this should be paged, since the list can be quite long.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The list of banned persons.
pub struct BannedPersonsResponse {
  pub banned: Vec<PersonView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response for a banned person.
pub struct BanPersonResponse {
  pub person_view: PersonView,
  pub banned: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Block a person.
pub struct BlockPerson {
  pub person_id: PersonId,
  pub block: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response for a person block.
pub struct BlockPersonResponse {
  pub person_view: PersonView,
  pub blocked: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Get your inbox (replies, comment mentions, post mentions, and messages)
pub struct ListInbox {
  #[cfg_attr(feature = "full", ts(optional))]
  pub type_: Option<InboxDataType>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub unread_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_cursor: Option<InboxCombinedPaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_back: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Get your inbox (replies, comment mentions, post mentions, and messages)
pub struct ListInboxResponse {
  pub inbox: Vec<InboxCombinedView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Mark a person mention as read.
pub struct MarkPersonCommentMentionAsRead {
  pub person_comment_mention_id: PersonCommentMentionId,
  pub read: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Mark a person mention as read.
pub struct MarkPersonPostMentionAsRead {
  pub person_post_mention_id: PersonPostMentionId,
  pub read: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Mark a comment reply as read.
pub struct MarkCommentReplyAsRead {
  pub comment_reply_id: CommentReplyId,
  pub read: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Delete your account.
pub struct DeleteAccount {
  pub password: SensitiveString,
  pub delete_content: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Reset your password via email.
pub struct PasswordReset {
  pub email: SensitiveString,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Change your password after receiving a reset request.
pub struct PasswordChangeAfterReset {
  pub token: SensitiveString,
  pub password: SensitiveString,
  pub password_verify: SensitiveString,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Get a count of the number of reports.
pub struct GetReportCount {
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_id: Option<CommunityId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response for the number of reports.
pub struct GetReportCountResponse {
  pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response containing a count of unread notifications.
pub struct GetUnreadCountResponse {
  pub count: i64,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Verify your email.
pub struct VerifyEmail {
  pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GenerateTotpSecretResponse {
  pub totp_secret_url: SensitiveString,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct UpdateTotp {
  pub totp_token: String,
  pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct UpdateTotpResponse {
  pub enabled: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Get your user's image / media uploads.
pub struct ListMedia {
  #[cfg_attr(feature = "full", ts(optional))]
  pub page: Option<i64>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ListMediaResponse {
  pub images: Vec<LocalImageView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ListLoginsResponse {
  pub logins: Vec<LoginToken>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Make a request to resend your verification email.
pub struct ResendVerificationEmail {
  pub email: SensitiveString,
}
