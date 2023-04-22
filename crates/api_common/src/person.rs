use crate::sensitive::Sensitive;
use lemmy_db_schema::{
  newtypes::{CommentReplyId, CommunityId, LanguageId, PersonId, PersonMentionId},
  CommentSortType,
  ListingType,
  SortType,
};
use lemmy_db_views::structs::{CommentView, PostView};
use lemmy_db_views_actor::structs::{
  CommentReplyView,
  CommunityModeratorView,
  PersonMentionView,
  PersonView,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct Login {
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub username_or_email: Sensitive<String>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub password: Sensitive<String>,
  pub totp_2fa_token: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct Register {
  pub username: String,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub password: Sensitive<String>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub password_verify: Sensitive<String>,
  pub show_nsfw: bool,
  /// email is mandatory if email verification is enabled on the server
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub email: Option<Sensitive<String>>,
  pub captcha_uuid: Option<String>,
  pub captcha_answer: Option<String>,
  pub honeypot: Option<String>,
  /// An answer is mandatory if require application is enabled on the server
  pub answer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetCaptcha {}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetCaptchaResponse {
  pub ok: Option<CaptchaResponse>, // Will be None if captchas are disabled
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct CaptchaResponse {
  /// A Base64 encoded png  
  pub png: String,
  /// A Base64 encoded wav audio  
  pub wav: String,
  pub uuid: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct SaveUserSettings {
  pub show_nsfw: Option<bool>,
  pub show_scores: Option<bool>,
  pub theme: Option<String>,
  pub default_sort_type: Option<SortType>,
  pub default_listing_type: Option<ListingType>,
  pub interface_language: Option<String>,
  pub avatar: Option<String>,
  pub banner: Option<String>,
  pub display_name: Option<String>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub email: Option<Sensitive<String>>,
  pub bio: Option<String>,
  pub matrix_user_id: Option<String>,
  pub show_avatars: Option<bool>,
  pub send_notifications_to_email: Option<bool>,
  pub bot_account: Option<bool>,
  pub show_bot_accounts: Option<bool>,
  pub show_read_posts: Option<bool>,
  pub show_new_post_notifs: Option<bool>,
  pub discussion_languages: Option<Vec<LanguageId>>,
  /// None leaves it as is, true will generate or regenerate it, false clears it out
  pub generate_totp_2fa: Option<bool>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ChangePassword {
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub new_password: Sensitive<String>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub new_password_verify: Sensitive<String>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub old_password: Sensitive<String>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct LoginResponse {
  /// This is None in response to `Register` if email verification is enabled, or the server requires registration applications.
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub jwt: Option<Sensitive<String>>,
  pub registration_created: bool,
  pub verify_email_sent: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetPersonDetails {
  pub person_id: Option<PersonId>, // One of these two are required
  /// Example: dessalines , or dessalines@xyz.tld
  pub username: Option<String>,
  pub sort: Option<SortType>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub community_id: Option<CommunityId>,
  pub saved_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Option<Sensitive<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetPersonDetailsResponse {
  pub person_view: PersonView,
  pub comments: Vec<CommentView>,
  pub posts: Vec<PostView>,
  pub moderates: Vec<CommunityModeratorView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetRepliesResponse {
  pub replies: Vec<CommentReplyView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetPersonMentionsResponse {
  pub mentions: Vec<PersonMentionView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct MarkAllAsRead {
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AddAdmin {
  pub person_id: PersonId,
  pub added: bool,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AddAdminResponse {
  pub admins: Vec<PersonView>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct BanPerson {
  pub person_id: PersonId,
  pub ban: bool,
  pub remove_data: Option<bool>,
  pub reason: Option<String>,
  pub expires: Option<i64>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetBannedPersons {
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct BannedPersonsResponse {
  pub banned: Vec<PersonView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct BanPersonResponse {
  pub person_view: PersonView,
  pub banned: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct BlockPerson {
  pub person_id: PersonId,
  pub block: bool,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct BlockPersonResponse {
  pub person_view: PersonView,
  pub blocked: bool,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetReplies {
  pub sort: Option<CommentSortType>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub unread_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetPersonMentions {
  pub sort: Option<CommentSortType>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub unread_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct MarkPersonMentionAsRead {
  pub person_mention_id: PersonMentionId,
  pub read: bool,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PersonMentionResponse {
  pub person_mention_view: PersonMentionView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct MarkCommentReplyAsRead {
  pub comment_reply_id: CommentReplyId,
  pub read: bool,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct CommentReplyResponse {
  pub comment_reply_view: CommentReplyView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct DeleteAccount {
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub password: Sensitive<String>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct DeleteAccountResponse {}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PasswordReset {
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub email: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PasswordResetResponse {}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PasswordChangeAfterReset {
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub token: Sensitive<String>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub password: Sensitive<String>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub password_verify: Sensitive<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetReportCount {
  pub community_id: Option<CommunityId>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetReportCountResponse {
  pub community_id: Option<CommunityId>,
  pub comment_reports: i64,
  pub post_reports: i64,
  pub private_message_reports: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetUnreadCount {
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetUnreadCountResponse {
  pub replies: i64,
  pub mentions: i64,
  pub private_messages: i64,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct VerifyEmail {
  pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct VerifyEmailResponse {}
