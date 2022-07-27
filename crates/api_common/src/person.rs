use crate::sensitive::Sensitive;
use lemmy_db_views::structs::{CommentView, PostView, PrivateMessageView};
use lemmy_db_views_actor::structs::{
  CommentReplyView,
  CommunityModeratorView,
  PersonMentionView,
  PersonViewSafe,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Login {
  pub username_or_email: Sensitive<String>,
  pub password: Sensitive<String>,
}
use lemmy_db_schema::{
  newtypes::{CommentReplyId, CommunityId, PersonId, PersonMentionId, PrivateMessageId},
  CommentSortType,
  SortType,
};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Register {
  pub username: String,
  pub password: Sensitive<String>,
  pub password_verify: Sensitive<String>,
  pub show_nsfw: bool,
  /// email is mandatory if email verification is enabled on the server
  pub email: Option<Sensitive<String>>,
  pub captcha_uuid: Option<String>,
  pub captcha_answer: Option<String>,
  pub honeypot: Option<String>,
  /// An answer is mandatory if require application is enabled on the server
  pub answer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetCaptcha {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetCaptchaResponse {
  pub ok: Option<CaptchaResponse>, // Will be None if captchas are disabled
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CaptchaResponse {
  pub png: String, // A Base64 encoded png
  pub wav: String, // A Base64 encoded wav audio
  pub uuid: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SaveUserSettings {
  pub show_nsfw: Option<bool>,
  pub show_scores: Option<bool>,
  pub theme: Option<String>,
  pub default_sort_type: Option<i16>,
  pub default_listing_type: Option<i16>,
  pub lang: Option<String>,
  pub avatar: Option<String>,
  pub banner: Option<String>,
  pub display_name: Option<String>,
  pub email: Option<Sensitive<String>>,
  pub bio: Option<String>,
  pub matrix_user_id: Option<String>,
  pub show_avatars: Option<bool>,
  pub send_notifications_to_email: Option<bool>,
  pub bot_account: Option<bool>,
  pub show_bot_accounts: Option<bool>,
  pub show_read_posts: Option<bool>,
  pub show_new_post_notifs: Option<bool>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChangePassword {
  pub new_password: Sensitive<String>,
  pub new_password_verify: Sensitive<String>,
  pub old_password: Sensitive<String>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginResponse {
  /// This is None in response to `Register` if email verification is enabled, or the server requires registration applications.
  pub jwt: Option<Sensitive<String>>,
  pub registration_created: bool,
  pub verify_email_sent: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetPersonDetails {
  pub person_id: Option<PersonId>, // One of these two are required
  /// Example: dessalines , or dessalines@xyz.tld
  pub username: Option<String>,
  pub sort: Option<SortType>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub community_id: Option<CommunityId>,
  pub saved_only: Option<bool>,
  pub auth: Option<Sensitive<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetPersonDetailsResponse {
  pub person_view: PersonViewSafe,
  pub comments: Vec<CommentView>,
  pub posts: Vec<PostView>,
  pub moderates: Vec<CommunityModeratorView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetRepliesResponse {
  pub replies: Vec<CommentReplyView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetPersonMentionsResponse {
  pub mentions: Vec<PersonMentionView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MarkAllAsRead {
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AddAdmin {
  pub person_id: PersonId,
  pub added: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddAdminResponse {
  pub admins: Vec<PersonViewSafe>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BanPerson {
  pub person_id: PersonId,
  pub ban: bool,
  pub remove_data: Option<bool>,
  pub reason: Option<String>,
  pub expires: Option<i64>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetBannedPersons {
  pub auth: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BannedPersonsResponse {
  pub banned: Vec<PersonViewSafe>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BanPersonResponse {
  pub person_view: PersonViewSafe,
  pub banned: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BlockPerson {
  pub person_id: PersonId,
  pub block: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockPersonResponse {
  pub person_view: PersonViewSafe,
  pub blocked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetReplies {
  pub sort: Option<CommentSortType>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub unread_only: Option<bool>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetPersonMentions {
  pub sort: Option<CommentSortType>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub unread_only: Option<bool>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MarkPersonMentionAsRead {
  pub person_mention_id: PersonMentionId,
  pub read: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonMentionResponse {
  pub person_mention_view: PersonMentionView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MarkCommentReplyAsRead {
  pub comment_reply_id: CommentReplyId,
  pub read: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentReplyResponse {
  pub comment_reply_view: CommentReplyView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DeleteAccount {
  pub password: Sensitive<String>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteAccountResponse {}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PasswordReset {
  pub email: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordResetResponse {}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PasswordChangeAfterReset {
  pub token: Sensitive<String>,
  pub password: Sensitive<String>,
  pub password_verify: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CreatePrivateMessage {
  pub content: String,
  pub recipient_id: PersonId,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EditPrivateMessage {
  pub private_message_id: PrivateMessageId,
  pub content: String,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DeletePrivateMessage {
  pub private_message_id: PrivateMessageId,
  pub deleted: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MarkPrivateMessageAsRead {
  pub private_message_id: PrivateMessageId,
  pub read: bool,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetPrivateMessages {
  pub unread_only: Option<bool>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivateMessagesResponse {
  pub private_messages: Vec<PrivateMessageView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivateMessageResponse {
  pub private_message_view: PrivateMessageView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetReportCount {
  pub community_id: Option<CommunityId>,
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetReportCountResponse {
  pub community_id: Option<CommunityId>,
  pub comment_reports: i64,
  pub post_reports: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GetUnreadCount {
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetUnreadCountResponse {
  pub replies: i64,
  pub mentions: i64,
  pub private_messages: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct VerifyEmail {
  pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VerifyEmailResponse {}
