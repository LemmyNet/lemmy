use crate::SiteView;
use chrono::{DateTime, Utc};
use lemmy_db_schema::{
  newtypes::{LanguageId, OAuthProviderId, PaginationCursor, TaglineId},
  source::{
    instance::Instance,
    language::Language,
    local_site_url_blocklist::LocalSiteUrlBlocklist,
    oauth_provider::{OAuthProvider, PublicOAuthProvider},
    tagline::Tagline,
  },
};
use lemmy_db_schema_file::enums::{
  CommentSortType,
  FederationMode,
  ListingType,
  PostListingMode,
  PostSortType,
  RegistrationMode,
};
use lemmy_db_views_api_misc::PluginMetadata;
use lemmy_db_views_person::PersonView;
use lemmy_db_views_readable_federation_state::ReadableFederationState;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminAllowInstanceParams {
  pub instance: String,
  pub allow: bool,
  pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminBlockInstanceParams {
  pub instance: String,
  pub block: bool,
  pub reason: Option<String>,
  pub expires_at: Option<DateTime<Utc>>,
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
  pub description: Option<String>,
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
  pub actor_name_max_length: Option<i32>,
  pub rate_limit_message: Option<i32>,
  pub rate_limit_message_per_second: Option<i32>,
  pub rate_limit_post: Option<i32>,
  pub rate_limit_post_per_second: Option<i32>,
  pub rate_limit_register: Option<i32>,
  pub rate_limit_register_per_second: Option<i32>,
  pub rate_limit_image: Option<i32>,
  pub rate_limit_image_per_second: Option<i32>,
  pub rate_limit_comment: Option<i32>,
  pub rate_limit_comment_per_second: Option<i32>,
  pub rate_limit_search: Option<i32>,
  pub rate_limit_search_per_second: Option<i32>,
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
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Delete an external auth method.
pub struct DeleteOAuthProvider {
  pub id: OAuthProviderId,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Delete a tagline
pub struct DeleteTagline {
  pub id: TaglineId,
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
  pub description: Option<String>,
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
  /// The max length of actor names.
  pub actor_name_max_length: Option<i32>,
  /// The number of messages allowed in a given time frame.
  pub rate_limit_message: Option<i32>,
  pub rate_limit_message_per_second: Option<i32>,
  /// The number of posts allowed in a given time frame.
  pub rate_limit_post: Option<i32>,
  pub rate_limit_post_per_second: Option<i32>,
  /// The number of registrations allowed in a given time frame.
  pub rate_limit_register: Option<i32>,
  pub rate_limit_register_per_second: Option<i32>,
  /// The number of image uploads allowed in a given time frame.
  pub rate_limit_image: Option<i32>,
  pub rate_limit_image_per_second: Option<i32>,
  /// The number of comments allowed in a given time frame.
  pub rate_limit_comment: Option<i32>,
  pub rate_limit_comment_per_second: Option<i32>,
  /// The number of searches allowed in a given time frame.
  pub rate_limit_search: Option<i32>,
  pub rate_limit_search_per_second: Option<i32>,
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A list of federated instances.
pub struct FederatedInstances {
  pub linked: Vec<InstanceWithFederationState>,
  pub allowed: Vec<InstanceWithFederationState>,
  pub blocked: Vec<InstanceWithFederationState>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A response of federated instances.
pub struct GetFederatedInstancesResponse {
  /// Optional, because federation may be disabled.
  pub federated_instances: Option<FederatedInstances>,
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
  pub admin_oauth_providers: Vec<OAuthProvider>,
  pub blocked_urls: Vec<LocalSiteUrlBlocklist>,
  // If true then uploads for post images or markdown images are disabled. Only avatars, icons and
  // banners can be set.
  pub image_upload_disabled: bool,
  pub active_plugins: Vec<PluginMetadata>,
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

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Fetches a list of taglines.
pub struct ListTaglines {
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A response for taglines.
pub struct ListTaglinesResponse {
  pub taglines: Vec<Tagline>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
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
pub struct TaglineResponse {
  pub tagline: Tagline,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Update a tagline
pub struct UpdateTagline {
  pub id: TaglineId,
  pub content: String,
}
