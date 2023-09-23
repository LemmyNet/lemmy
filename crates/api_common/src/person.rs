use crate::sensitive::Sensitive;
use lemmy_db_schema::{
  newtypes::{CommentReplyId, CommunityId, LanguageId, PersonId, PersonMentionId},
  CommentSortType, ListingType, SortType,
};
use lemmy_db_views::structs::{CommentView, PostView};
use lemmy_db_views_actor::structs::{
  CommentReplyView, CommunityModeratorView, PersonMentionView, PersonView,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Logging into lemmy.
pub struct Login {
  pub username_or_email: Sensitive<String>,
  pub password: Sensitive<String>,
  /// May be required, if totp is enabled for their account.
  pub totp_2fa_token: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Register / Sign up to lemmy.
pub struct Register {
  pub username: String,
  pub password: Sensitive<String>,
  pub password_verify: Sensitive<String>,
  pub show_nsfw: bool,
  /// email is mandatory if email verification is enabled on the server
  pub email: Option<Sensitive<String>>,
  /// The UUID of the captcha item.
  pub captcha_uuid: Option<String>,
  /// Your captcha answer.
  pub captcha_answer: Option<String>,
  /// A form field to trick signup bots. Should be None.
  pub honeypot: Option<String>,
  /// An answer is mandatory if require application is enabled on the server
  pub answer: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A wrapper for the captcha response.
pub struct GetCaptchaResponse {
  /// Will be None if captchas are disabled.
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
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Saves settings for your user.
pub struct SaveUserSettings {
  /// Show nsfw posts.
  pub show_nsfw: Option<bool>,
  pub blur_nsfw: Option<bool>,
  pub auto_expand: Option<bool>,
  /// Show post and comment scores.
  pub show_scores: Option<bool>,
  /// Your user's theme.
  pub theme: Option<String>,
  pub default_sort_type: Option<SortType>,
  pub default_listing_type: Option<ListingType>,
  /// The language of the lemmy interface
  pub interface_language: Option<String>,
  /// A URL for your avatar.
  pub avatar: Option<String>,
  /// A URL for your banner.
  pub banner: Option<String>,
  /// Your display name, which can contain strange characters, and does not need to be unique.
  pub display_name: Option<String>,
  /// Your email.
  pub email: Option<Sensitive<String>>,
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
  /// Whether to show notifications for new posts.
  // TODO notifs need to be reworked.
  pub show_new_post_notifs: Option<bool>,
  /// A list of languages you are able to see discussion in.
  pub discussion_languages: Option<Vec<LanguageId>>,
  /// Open links in a new tab
  pub open_links_in_new_tab: Option<bool>,
  /// Enable infinite scroll
  pub infinite_scroll_enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Changes your account password.
pub struct ChangePassword {
  pub new_password: Sensitive<String>,
  pub new_password_verify: Sensitive<String>,
  pub old_password: Sensitive<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response for your login.
pub struct LoginResponse {
  /// This is None in response to `Register` if email verification is enabled, or the server requires registration applications.
  pub jwt: Option<Sensitive<String>>,
  /// If registration applications are required, this will return true for a signup response.
  pub registration_created: bool,
  /// If email verifications are required, this will return true for a signup response.
  pub verify_email_sent: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Gets a person's details.
///
/// Either person_id, or username are required.
pub struct GetPersonDetails {
  pub person_id: Option<PersonId>,
  /// Example: dessalines , or dessalines@xyz.tld
  pub username: Option<String>,
  pub sort: Option<SortType>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub community_id: Option<CommunityId>,
  pub saved_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A person's details response.
pub struct GetPersonDetailsResponse {
  pub person_view: PersonView,
  pub comments: Vec<CommentView>,
  pub posts: Vec<PostView>,
  pub moderates: Vec<CommunityModeratorView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
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
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Ban a person from the site.
pub struct BanPerson {
  pub person_id: PersonId,
  pub ban: bool,
  /// Optionally remove all their data. Useful for new troll accounts.
  pub remove_data: Option<bool>,
  pub reason: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
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
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Get comment replies.
pub struct GetReplies {
  pub sort: Option<CommentSortType>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub unread_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Fetches your replies.
// TODO, replies and mentions below should be redone as tagged enums.
pub struct GetRepliesResponse {
  pub replies: Vec<CommentReplyView>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Get mentions for your user.
pub struct GetPersonMentions {
  pub sort: Option<CommentSortType>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub unread_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response of mentions for your user.
pub struct GetPersonMentionsResponse {
  pub mentions: Vec<PersonMentionView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Mark a person mention as read.
pub struct MarkPersonMentionAsRead {
  pub person_mention_id: PersonMentionId,
  pub read: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response for a person mention action.
pub struct PersonMentionResponse {
  pub person_mention_view: PersonMentionView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Mark a comment reply as read.
pub struct MarkCommentReplyAsRead {
  pub comment_reply_id: CommentReplyId,
  pub read: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response for a comment reply action.
pub struct CommentReplyResponse {
  pub comment_reply_view: CommentReplyView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Delete your account.
pub struct DeleteAccount {
  pub password: Sensitive<String>,
  pub delete_content: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response of deleting your account.
pub struct DeleteAccountResponse {}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Reset your password via email.
pub struct PasswordReset {
  pub email: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response of a password reset.
pub struct PasswordResetResponse {}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Change your password after receiving a reset request.
pub struct PasswordChangeAfterReset {
  pub token: Sensitive<String>,
  pub password: Sensitive<String>,
  pub password_verify: Sensitive<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Get a count of the number of reports.
pub struct GetReportCount {
  pub community_id: Option<CommunityId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response for the number of reports.
pub struct GetReportCountResponse {
  pub community_id: Option<CommunityId>,
  pub comment_reports: i64,
  pub post_reports: i64,
  pub private_message_reports: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response containing counts for your notifications.
pub struct GetUnreadCountResponse {
  pub replies: i64,
  pub mentions: i64,
  pub private_messages: i64,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Verify your email.
pub struct VerifyEmail {
  pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response to verifying your email.
pub struct VerifyEmailResponse {}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GenerateTotpSecretResponse {
  pub totp_secret_url: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
