use lemmy_db_views::{
  comment_view::CommentView,
  post_view::PostView,
  private_message_view::PrivateMessageView,
};
use lemmy_db_views_actor::{
  community_follower_view::CommunityFollowerView,
  community_moderator_view::CommunityModeratorView,
  user_mention_view::UserMentionView,
  user_view::UserViewSafe,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct Login {
  pub username_or_email: String,
  pub password: String,
}

#[derive(Deserialize)]
pub struct Register {
  pub username: String,
  pub email: Option<String>,
  pub password: String,
  pub password_verify: String,
  pub show_nsfw: bool,
  pub captcha_uuid: Option<String>,
  pub captcha_answer: Option<String>,
}

#[derive(Deserialize)]
pub struct GetCaptcha {}

#[derive(Serialize)]
pub struct GetCaptchaResponse {
  pub ok: Option<CaptchaResponse>, // Will be None if captchas are disabled
}

#[derive(Serialize)]
pub struct CaptchaResponse {
  pub png: String,         // A Base64 encoded png
  pub wav: Option<String>, // A Base64 encoded wav audio
  pub uuid: String,
}

#[derive(Deserialize)]
pub struct SaveUserSettings {
  pub show_nsfw: bool,
  pub theme: String,
  pub default_sort_type: i16,
  pub default_listing_type: i16,
  pub lang: String,
  pub avatar: Option<String>,
  pub banner: Option<String>,
  pub preferred_username: Option<String>,
  pub email: Option<String>,
  pub bio: Option<String>,
  pub matrix_user_id: Option<String>,
  pub new_password: Option<String>,
  pub new_password_verify: Option<String>,
  pub old_password: Option<String>,
  pub show_avatars: bool,
  pub send_notifications_to_email: bool,
  pub auth: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
  pub jwt: String,
}

#[derive(Deserialize)]
pub struct GetUserDetails {
  pub user_id: Option<i32>,
  pub username: Option<String>,
  pub sort: String,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub community_id: Option<i32>,
  pub saved_only: bool,
  pub auth: Option<String>,
}

#[derive(Serialize)]
pub struct GetUserDetailsResponse {
  pub user_view: UserViewSafe,
  pub follows: Vec<CommunityFollowerView>,
  pub moderates: Vec<CommunityModeratorView>,
  pub comments: Vec<CommentView>,
  pub posts: Vec<PostView>,
}

#[derive(Serialize)]
pub struct GetRepliesResponse {
  pub replies: Vec<CommentView>,
}

#[derive(Serialize)]
pub struct GetUserMentionsResponse {
  pub mentions: Vec<UserMentionView>,
}

#[derive(Deserialize)]
pub struct MarkAllAsRead {
  pub auth: String,
}

#[derive(Deserialize)]
pub struct AddAdmin {
  pub user_id: i32,
  pub added: bool,
  pub auth: String,
}

#[derive(Serialize, Clone)]
pub struct AddAdminResponse {
  pub admins: Vec<UserViewSafe>,
}

#[derive(Deserialize)]
pub struct BanUser {
  pub user_id: i32,
  pub ban: bool,
  pub remove_data: bool,
  pub reason: Option<String>,
  pub expires: Option<i64>,
  pub auth: String,
}

#[derive(Serialize, Clone)]
pub struct BanUserResponse {
  pub user_view: UserViewSafe,
  pub banned: bool,
}

#[derive(Deserialize)]
pub struct GetReplies {
  pub sort: String,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub unread_only: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct GetUserMentions {
  pub sort: String,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub unread_only: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct MarkUserMentionAsRead {
  pub user_mention_id: i32,
  pub read: bool,
  pub auth: String,
}

#[derive(Serialize, Clone)]
pub struct UserMentionResponse {
  pub user_mention_view: UserMentionView,
}

#[derive(Deserialize)]
pub struct DeleteAccount {
  pub password: String,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct PasswordReset {
  pub email: String,
}

#[derive(Serialize, Clone)]
pub struct PasswordResetResponse {}

#[derive(Deserialize)]
pub struct PasswordChange {
  pub token: String,
  pub password: String,
  pub password_verify: String,
}

#[derive(Deserialize)]
pub struct CreatePrivateMessage {
  pub content: String,
  pub recipient_id: i32,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct EditPrivateMessage {
  pub private_message_id: i32,
  pub content: String,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct DeletePrivateMessage {
  pub private_message_id: i32,
  pub deleted: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct MarkPrivateMessageAsRead {
  pub private_message_id: i32,
  pub read: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct GetPrivateMessages {
  pub unread_only: bool,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: String,
}

#[derive(Serialize, Clone)]
pub struct PrivateMessagesResponse {
  pub private_messages: Vec<PrivateMessageView>,
}

#[derive(Serialize, Clone)]
pub struct PrivateMessageResponse {
  pub private_message_view: PrivateMessageView,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetReportCount {
  pub community: Option<i32>,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetReportCountResponse {
  pub community: Option<i32>,
  pub comment_reports: i64,
  pub post_reports: i64,
}
