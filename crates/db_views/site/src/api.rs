use crate::{ReadableFederationState, SiteView};
use lemmy_db_schema::{
  newtypes::{LanguageId, MultiCommunityId, OAuthProviderId, TaglineId},
  source::{
    comment::Comment,
    community::Community,
    instance::Instance,
    language::Language,
    local_site_url_blocklist::LocalSiteUrlBlocklist,
    local_user::LocalUser,
    login_token::LoginToken,
    oauth_provider::{AdminOAuthProvider, PublicOAuthProvider},
    person::Person,
    post::Post,
    private_message::PrivateMessage,
    tagline::Tagline,
  },
};
use lemmy_db_schema_file::{
  InstanceId,
  enums::{
    CommentSortType,
    FederationMode,
    ListingType,
    PostListingMode,
    PostSortType,
    RegistrationMode,
    VoteShow,
  },
};
use lemmy_db_views_community::MultiCommunityView;
use lemmy_db_views_community_follower::CommunityFollowerView;
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::PersonView;
use lemmy_diesel_utils::{pagination::PaginationCursor, sensitive::SensitiveString};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;
#[cfg(feature = "full")]
use {extism::FromBytes, extism_convert::Json};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminAllowInstanceParams {
  pub instance: String,
  pub allow: bool,
  pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminBlockInstanceParams {
  pub instance: String,
  pub block: bool,
  pub reason: String,
  /// A time that the block will expire, in unix epoch seconds.
  ///
  /// An i64 unix timestamp is used for a simpler API client implementation.
  pub expires_at: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Logging in with an OAuth 2.0 authorization
pub struct AuthenticateWithOauth {
  pub code: String,
  pub oauth_provider_id: OAuthProviderId,
  pub redirect_uri: Url,
  pub show_nsfw: Option<bool>,
  /// Username is mandatory at registration time
  pub username: Option<String>,
  /// An answer is mandatory if require application is enabled on the server
  pub answer: Option<String>,
  pub pkce_code_verifier: Option<String>,
  /// If this is true the login is valid forever, otherwise it expires after one week.
  pub stay_logged_in: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create an external auth method.
pub struct CreateOAuthProvider {
  pub display_name: String,
  pub issuer: String,
  pub authorization_endpoint: String,
  pub token_endpoint: String,
  pub userinfo_endpoint: String,
  pub id_claim: String,
  pub client_id: String,
  pub client_secret: String,
  pub scopes: String,
  pub auto_verify_email: Option<bool>,
  pub account_linking_enabled: Option<bool>,
  pub use_pkce: Option<bool>,
  pub enabled: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Creates a site. Should be done after first running lemmy.
pub struct CreateSite {
  pub name: String,
  pub sidebar: Option<String>,
  pub summary: Option<String>,
  pub community_creation_admin_only: Option<bool>,
  pub require_email_verification: Option<bool>,
  pub application_question: Option<String>,
  pub private_instance: Option<bool>,
  pub default_theme: Option<String>,
  pub default_post_listing_type: Option<ListingType>,
  pub default_post_listing_mode: Option<PostListingMode>,
  pub default_post_sort_type: Option<PostSortType>,
  pub default_post_time_range_seconds: Option<i32>,
  pub default_comment_sort_type: Option<CommentSortType>,
  pub legal_information: Option<String>,
  pub application_email_admins: Option<bool>,
  pub discussion_languages: Option<Vec<LanguageId>>,
  pub slur_filter_regex: Option<String>,
  pub rate_limit_message_max_requests: Option<i32>,
  pub rate_limit_message_interval_seconds: Option<i32>,
  pub rate_limit_post_max_requests: Option<i32>,
  pub rate_limit_post_interval_seconds: Option<i32>,
  pub rate_limit_register_max_requests: Option<i32>,
  pub rate_limit_register_interval_seconds: Option<i32>,
  pub rate_limit_image_max_requests: Option<i32>,
  pub rate_limit_image_interval_seconds: Option<i32>,
  pub rate_limit_comment_max_requests: Option<i32>,
  pub rate_limit_comment_interval_seconds: Option<i32>,
  pub rate_limit_search_max_requests: Option<i32>,
  pub rate_limit_search_interval_seconds: Option<i32>,
  pub rate_limit_import_user_settings_max_requests: Option<i32>,
  pub rate_limit_import_user_settings_interval_seconds: Option<i32>,
  pub federation_enabled: Option<bool>,
  pub captcha_enabled: Option<bool>,
  pub captcha_difficulty: Option<String>,
  pub registration_mode: Option<RegistrationMode>,
  pub oauth_registration: Option<bool>,
  pub content_warning: Option<String>,
  pub post_upvotes: Option<FederationMode>,
  pub post_downvotes: Option<FederationMode>,
  pub comment_upvotes: Option<FederationMode>,
  pub comment_downvotes: Option<FederationMode>,
  pub disallow_nsfw_content: Option<bool>,
  pub disable_email_notifications: Option<bool>,
  pub suggested_communities: Option<MultiCommunityId>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Delete an external auth method.
pub struct DeleteOAuthProvider {
  pub id: OAuthProviderId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Edit an external auth method.
pub struct EditOAuthProvider {
  pub id: OAuthProviderId,
  pub display_name: Option<String>,
  pub authorization_endpoint: Option<String>,
  pub token_endpoint: Option<String>,
  pub userinfo_endpoint: Option<String>,
  pub id_claim: Option<String>,
  pub client_secret: Option<String>,
  pub scopes: Option<String>,
  pub auto_verify_email: Option<bool>,
  pub account_linking_enabled: Option<bool>,
  pub use_pkce: Option<bool>,
  pub enabled: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Edits a site.
pub struct EditSite {
  pub name: Option<String>,
  /// A sidebar for the site, in markdown.
  pub sidebar: Option<String>,
  /// A shorter, one line description of your site.
  pub summary: Option<String>,
  /// Limits community creation to admins only.
  pub community_creation_admin_only: Option<bool>,
  /// Whether to require email verification.
  pub require_email_verification: Option<bool>,
  /// Your application question form. This is in markdown, and can be many questions.
  pub application_question: Option<String>,
  /// Whether your instance is public, or private.
  pub private_instance: Option<bool>,
  /// The default theme. Usually "browser"
  pub default_theme: Option<String>,
  /// The default post listing type, usually "local"
  pub default_post_listing_type: Option<ListingType>,
  /// Default value for listing mode, usually "list"
  pub default_post_listing_mode: Option<PostListingMode>,
  /// The default post sort, usually "active"
  pub default_post_sort_type: Option<PostSortType>,
  /// A default time range limit to apply to post sorts, in seconds. 0 means none.
  pub default_post_time_range_seconds: Option<i32>,
  /// A default fetch limit for number of items returned.
  pub default_items_per_page: Option<i32>,
  /// The default comment sort, usually "hot"
  pub default_comment_sort_type: Option<CommentSortType>,
  /// An optional page of legal information
  pub legal_information: Option<String>,
  /// Whether to email admins when receiving a new application.
  pub application_email_admins: Option<bool>,
  /// A list of allowed discussion languages.
  pub discussion_languages: Option<Vec<LanguageId>>,
  /// A regex string of items to filter.
  pub slur_filter_regex: Option<String>,
  /// The number of messages allowed in a given time frame.
  pub rate_limit_message_max_requests: Option<i32>,
  pub rate_limit_message_interval_seconds: Option<i32>,
  /// The number of posts allowed in a given time frame.
  pub rate_limit_post_max_requests: Option<i32>,
  pub rate_limit_post_interval_seconds: Option<i32>,
  /// The number of registrations allowed in a given time frame.
  pub rate_limit_register_max_requests: Option<i32>,
  pub rate_limit_register_interval_seconds: Option<i32>,
  /// The number of image uploads allowed in a given time frame.
  pub rate_limit_image_max_requests: Option<i32>,
  pub rate_limit_image_interval_seconds: Option<i32>,
  /// The number of comments allowed in a given time frame.
  pub rate_limit_comment_max_requests: Option<i32>,
  pub rate_limit_comment_interval_seconds: Option<i32>,
  /// The number of searches allowed in a given time frame.
  pub rate_limit_search_max_requests: Option<i32>,
  pub rate_limit_search_interval_seconds: Option<i32>,
  /// The number of settings imports or exports allowed in a given time frame.
  pub rate_limit_import_user_settings_max_requests: Option<i32>,
  pub rate_limit_import_user_settings_interval_seconds: Option<i32>,
  /// Whether to enable federation.
  pub federation_enabled: Option<bool>,
  /// Whether to enable captchas for signups.
  pub captcha_enabled: Option<bool>,
  /// The captcha difficulty. Can be easy, medium, or hard
  pub captcha_difficulty: Option<String>,
  /// A list of blocked URLs
  pub blocked_urls: Option<Vec<String>>,
  pub registration_mode: Option<RegistrationMode>,
  /// Whether to email admins for new reports.
  pub reports_email_admins: Option<bool>,
  /// If present, nsfw content is visible by default. Should be displayed by frontends/clients
  /// when the site is first opened by a user.
  pub content_warning: Option<String>,
  /// Whether or not external auth methods can auto-register users.
  pub oauth_registration: Option<bool>,
  /// What kind of post upvotes your site allows.
  pub post_upvotes: Option<FederationMode>,
  /// What kind of post downvotes your site allows.
  pub post_downvotes: Option<FederationMode>,
  /// What kind of comment upvotes your site allows.
  pub comment_upvotes: Option<FederationMode>,
  /// What kind of comment downvotes your site allows.
  pub comment_downvotes: Option<FederationMode>,
  /// Block NSFW content being created
  pub disallow_nsfw_content: Option<bool>,
  /// Dont send email notifications to users for new replies, mentions etc
  pub disable_email_notifications: Option<bool>,
  /// A multicommunity with suggested communities which is shown on the homepage
  pub suggested_communities: Option<MultiCommunityId>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub enum GetFederatedInstancesKind {
  #[default]
  All,
  Linked,
  Allowed,
  Blocked,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct GetFederatedInstances {
  pub domain_filter: Option<String>,
  pub kind: GetFederatedInstancesKind,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// An expanded response for a site.
pub struct GetSiteResponse {
  pub site_view: SiteView,
  pub admins: Vec<PersonView>,
  pub version: String,
  pub all_languages: Vec<Language>,
  pub discussion_languages: Vec<LanguageId>,
  /// If the site has any taglines, a random one is included here for displaying
  pub tagline: Option<Tagline>,
  /// A list of external auth methods your site supports.
  pub oauth_providers: Vec<PublicOAuthProvider>,
  pub admin_oauth_providers: Vec<AdminOAuthProvider>,
  pub blocked_urls: Vec<LocalSiteUrlBlocklist>,
  // If true then uploads for post images or markdown images are disabled. Only avatars, icons and
  // banners can be set.
  pub image_upload_disabled: bool,
  pub active_plugins: Vec<PluginMetadata>,
  /// The number of seconds between the last application published, and approved / denied time.
  ///
  /// Useful for estimating when your application will be approved.
  pub last_application_duration_seconds: Option<i64>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct InstanceWithFederationState {
  #[serde(flatten)]
  pub instance: Instance,
  /// if federation to this instance is or was active, show state of outgoing federation to this
  /// instance
  pub federation_state: Option<ReadableFederationState>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The response for a site.
pub struct SiteResponse {
  pub site_view: SiteView,
}

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
  /// If this is true the login is valid forever, otherwise it expires after one week.
  pub stay_logged_in: Option<bool>,
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
/// Logging into lemmy.
///
/// Note: Banned users can still log in, to be able to do certain things like delete
/// their account.
pub struct Login {
  pub username_or_email: SensitiveString,
  pub password: SensitiveString,
  /// May be required, if totp is enabled for their account.
  pub totp_2fa_token: Option<String>,
  /// If this is true the login is valid forever, otherwise it expires after one week.
  pub stay_logged_in: Option<bool>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Your user info.
pub struct MyUserInfo {
  pub local_user_view: LocalUserView,
  pub follows: Vec<CommunityFollowerView>,
  pub moderates: Vec<CommunityModeratorView>,
  pub multi_community_follows: Vec<MultiCommunityView>,
  pub community_blocks: Vec<Community>,
  pub instance_communities_blocks: Vec<Instance>,
  pub instance_persons_blocks: Vec<Instance>,
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

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Make a request to resend your verification email.
pub struct ResendVerificationEmail {
  pub email: SensitiveString,
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
  /// A default fetch limit for number of items returned.
  pub default_items_per_page: Option<i32>,
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
  /// Whether user avatars or inline images in the UI that are gifs should be allowed to play or
  /// should be paused
  pub enable_animated_images: Option<bool>,
  /// Whether a user can send / receive private messages
  pub enable_private_messages: Option<bool>,
  /// Whether to auto-collapse bot comments.
  pub collapse_bot_comments: Option<bool>,
  /// Some vote display mode settings
  pub show_score: Option<bool>,
  pub show_upvotes: Option<bool>,
  pub show_downvotes: Option<VoteShow>,
  pub show_upvote_percentage: Option<bool>,
  /// Whether to automatically mark fetched posts as read.
  pub auto_mark_fetched_posts_as_read: Option<bool>,
  /// Whether to hide posts containing images/videos.
  pub hide_media: Option<bool>,
  /// Whether to show vote totals given to others.
  pub show_person_votes: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct EditTotp {
  pub totp_token: String,
  pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct EditTotpResponse {
  pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Block an instance's persons.
pub struct UserBlockInstancePersonsParams {
  pub instance_id: InstanceId,
  pub block: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Block an instance's communities.
pub struct UserBlockInstanceCommunitiesParams {
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a tagline
pub struct CreateTagline {
  pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Delete a tagline
pub struct DeleteTagline {
  pub id: TaglineId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Fetches a list of taglines.
pub struct ListTaglines {
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct TaglineResponse {
  pub tagline: Tagline,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Update a tagline
pub struct EditTagline {
  pub id: TaglineId,
  pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(FromBytes))]
#[cfg_attr(feature = "full", encoding(Json))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct PluginMetadata {
  pub name: String,
  pub url: Option<Url>,
  pub description: Option<String>,
}

impl PluginMetadata {
  pub fn new(name: &'static str, url: &'static str, description: &'static str) -> Self {
    Self {
      name: name.to_string(),
      url: url.parse().ok(),
      description: Some(description.to_string()),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Does an apub fetch for an object.
pub struct ResolveObject {
  /// Can be the full url, or a shortened version like: !fediverse@lemmy.ml
  pub q: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(tag = "type_", rename_all = "snake_case")]
pub enum PostOrCommentOrPrivateMessage {
  Post(Post),
  Comment(Comment),
  PrivateMessage(PrivateMessage),
}

/// Backup of user data. This struct should never be changed so that the data can be used as a
/// long-term backup in case the instance goes down unexpectedly. All fields are optional to allow
/// importing partial backups.
///
/// This data should not be parsed by apps/clients, but directly downloaded as a file.
///
/// Be careful with any changes to this struct, to avoid breaking changes which could prevent
/// importing older backups.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct UserSettingsBackup {
  pub display_name: Option<String>,
  pub bio: Option<String>,
  pub avatar: Option<Url>,
  pub banner: Option<Url>,
  pub matrix_id: Option<String>,
  pub bot_account: Option<bool>,
  // TODO: might be worth making a separate struct for settings backup, to avoid breakage in case
  //       fields are renamed, and to avoid storing unnecessary fields like person_id or email
  pub settings: Option<LocalUser>,
  #[serde(default)]
  pub followed_communities: Vec<Url>,
  #[serde(default)]
  pub saved_posts: Vec<Url>,
  #[serde(default)]
  pub saved_comments: Vec<Url>,
  #[serde(default)]
  pub blocked_communities: Vec<Url>,
  #[serde(default)]
  pub blocked_users: Vec<Url>,
  #[serde(default)]
  #[serde(alias = "blocked_instances")] // the name used by v0.19
  pub blocked_instances_communities: Vec<String>,
  #[serde(default)]
  pub blocked_instances_persons: Vec<String>,
  #[serde(default)]
  pub blocking_keywords: Vec<String>,
  #[serde(default)]
  pub discussion_languages: Vec<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Your exported data.
pub struct ExportDataResponse {
  pub notifications: Vec<PostOrCommentOrPrivateMessage>,
  pub content: Vec<PostOrCommentOrPrivateMessage>,
  pub read_posts: Vec<Url>,
  pub liked: Vec<Url>,
  pub moderates: Vec<Url>,
  pub settings: UserSettingsBackup,
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

/// Contains the amount of unread items of various types. For normal users this means the number of
/// unread notifications, mods and admins get additional unread counts for reports, registration
/// applications and pending follows to private communities.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct UnreadCountsResponse {
  pub notification_count: i64,
  pub report_count: Option<i64>,
  pub pending_follow_count: Option<i64>,
  pub registration_application_count: Option<i64>,
}
