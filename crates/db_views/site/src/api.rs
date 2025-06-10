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
use lemmy_db_views_api_misc::{MyUserInfo, PluginMetadata};
use lemmy_db_views_person::PersonView;
use lemmy_db_views_readable_federation_state::ReadableFederationState;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AdminAllowInstanceParams {
  pub instance: String,
  pub allow: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AdminBlockInstanceParams {
  pub instance: String,
  pub block: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub expires_at: Option<DateTime<Utc>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Logging in with an OAuth 2.0 authorization
pub struct AuthenticateWithOauth {
  pub code: String,
  pub oauth_provider_id: OAuthProviderId,
  pub redirect_uri: Url,
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_nsfw: Option<bool>,
  /// Username is mandatory at registration time
  #[cfg_attr(feature = "full", ts(optional))]
  pub username: Option<String>,
  /// An answer is mandatory if require application is enabled on the server
  #[cfg_attr(feature = "full", ts(optional))]
  pub answer: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub pkce_code_verifier: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
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
  #[cfg_attr(feature = "full", ts(optional))]
  pub auto_verify_email: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub account_linking_enabled: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub use_pkce: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub enabled: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Creates a site. Should be done after first running lemmy.
pub struct CreateSite {
  pub name: String,
  #[cfg_attr(feature = "full", ts(optional))]
  pub sidebar: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub description: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_creation_admin_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub require_email_verification: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub application_question: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub private_instance: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_theme: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_listing_type: Option<ListingType>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_listing_mode: Option<PostListingMode>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_sort_type: Option<PostSortType>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_time_range_seconds: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_comment_sort_type: Option<CommentSortType>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub legal_information: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub application_email_admins: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub discussion_languages: Option<Vec<LanguageId>>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub slur_filter_regex: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub actor_name_max_length: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_message: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_message_per_second: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_post: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_post_per_second: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_register: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_register_per_second: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_image: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_image_per_second: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_comment: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_comment_per_second: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_search: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_search_per_second: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub federation_enabled: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub captcha_enabled: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub captcha_difficulty: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub registration_mode: Option<RegistrationMode>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub oauth_registration: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub content_warning: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_upvotes: Option<FederationMode>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_downvotes: Option<FederationMode>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_upvotes: Option<FederationMode>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_downvotes: Option<FederationMode>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub disallow_nsfw_content: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub disable_email_notifications: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Delete an external auth method.
pub struct DeleteOAuthProvider {
  pub id: OAuthProviderId,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Delete a tagline
pub struct DeleteTagline {
  pub id: TaglineId,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Edit an external auth method.
pub struct EditOAuthProvider {
  pub id: OAuthProviderId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub display_name: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub authorization_endpoint: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub token_endpoint: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub userinfo_endpoint: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub id_claim: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub client_secret: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub scopes: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub auto_verify_email: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub account_linking_enabled: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub use_pkce: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub enabled: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Edits a site.
pub struct EditSite {
  #[cfg_attr(feature = "full", ts(optional))]
  pub name: Option<String>,
  /// A sidebar for the site, in markdown.
  #[cfg_attr(feature = "full", ts(optional))]
  pub sidebar: Option<String>,
  /// A shorter, one line description of your site.
  #[cfg_attr(feature = "full", ts(optional))]
  pub description: Option<String>,
  /// Limits community creation to admins only.
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_creation_admin_only: Option<bool>,
  /// Whether to require email verification.
  #[cfg_attr(feature = "full", ts(optional))]
  pub require_email_verification: Option<bool>,
  /// Your application question form. This is in markdown, and can be many questions.
  #[cfg_attr(feature = "full", ts(optional))]
  pub application_question: Option<String>,
  /// Whether your instance is public, or private.
  #[cfg_attr(feature = "full", ts(optional))]
  pub private_instance: Option<bool>,
  /// The default theme. Usually "browser"
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_theme: Option<String>,
  /// The default post listing type, usually "local"
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_listing_type: Option<ListingType>,
  /// Default value for listing mode, usually "list"
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_listing_mode: Option<PostListingMode>,
  /// The default post sort, usually "active"
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_sort_type: Option<PostSortType>,
  /// A default time range limit to apply to post sorts, in seconds. 0 means none.
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_post_time_range_seconds: Option<i32>,
  /// The default comment sort, usually "hot"
  #[cfg_attr(feature = "full", ts(optional))]
  pub default_comment_sort_type: Option<CommentSortType>,
  /// An optional page of legal information
  #[cfg_attr(feature = "full", ts(optional))]
  pub legal_information: Option<String>,
  /// Whether to email admins when receiving a new application.
  #[cfg_attr(feature = "full", ts(optional))]
  pub application_email_admins: Option<bool>,
  /// A list of allowed discussion languages.
  #[cfg_attr(feature = "full", ts(optional))]
  pub discussion_languages: Option<Vec<LanguageId>>,
  /// A regex string of items to filter.
  #[cfg_attr(feature = "full", ts(optional))]
  pub slur_filter_regex: Option<String>,
  /// The max length of actor names.
  #[cfg_attr(feature = "full", ts(optional))]
  pub actor_name_max_length: Option<i32>,
  /// The number of messages allowed in a given time frame.
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_message: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_message_per_second: Option<i32>,
  /// The number of posts allowed in a given time frame.
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_post: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_post_per_second: Option<i32>,
  /// The number of registrations allowed in a given time frame.
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_register: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_register_per_second: Option<i32>,
  /// The number of image uploads allowed in a given time frame.
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_image: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_image_per_second: Option<i32>,
  /// The number of comments allowed in a given time frame.
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_comment: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_comment_per_second: Option<i32>,
  /// The number of searches allowed in a given time frame.
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_search: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub rate_limit_search_per_second: Option<i32>,
  /// Whether to enable federation.
  #[cfg_attr(feature = "full", ts(optional))]
  pub federation_enabled: Option<bool>,
  /// Whether to enable captchas for signups.
  #[cfg_attr(feature = "full", ts(optional))]
  pub captcha_enabled: Option<bool>,
  /// The captcha difficulty. Can be easy, medium, or hard
  #[cfg_attr(feature = "full", ts(optional))]
  pub captcha_difficulty: Option<String>,
  /// A list of blocked URLs
  #[cfg_attr(feature = "full", ts(optional))]
  pub blocked_urls: Option<Vec<String>>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub registration_mode: Option<RegistrationMode>,
  /// Whether to email admins for new reports.
  #[cfg_attr(feature = "full", ts(optional))]
  pub reports_email_admins: Option<bool>,
  /// If present, nsfw content is visible by default. Should be displayed by frontends/clients
  /// when the site is first opened by a user.
  #[cfg_attr(feature = "full", ts(optional))]
  pub content_warning: Option<String>,
  /// Whether or not external auth methods can auto-register users.
  #[cfg_attr(feature = "full", ts(optional))]
  pub oauth_registration: Option<bool>,
  /// What kind of post upvotes your site allows.
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_upvotes: Option<FederationMode>,
  /// What kind of post downvotes your site allows.
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_downvotes: Option<FederationMode>,
  /// What kind of comment upvotes your site allows.
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_upvotes: Option<FederationMode>,
  /// What kind of comment downvotes your site allows.
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_downvotes: Option<FederationMode>,
  /// Block NSFW content being created
  #[cfg_attr(feature = "full", ts(optional))]
  pub disallow_nsfw_content: Option<bool>,
  /// Dont send email notifications to users for new replies, mentions etc
  #[cfg_attr(feature = "full", ts(optional))]
  pub disable_email_notifications: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A list of federated instances.
pub struct FederatedInstances {
  pub linked: Vec<InstanceWithFederationState>,
  pub allowed: Vec<InstanceWithFederationState>,
  pub blocked: Vec<InstanceWithFederationState>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response of federated instances.
pub struct GetFederatedInstancesResponse {
  /// Optional, because federation may be disabled.
  #[cfg_attr(feature = "full", ts(optional))]
  pub federated_instances: Option<FederatedInstances>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// An expanded response for a site.
pub struct GetSiteResponse {
  pub site_view: SiteView,
  pub admins: Vec<PersonView>,
  pub version: String,
  #[cfg_attr(feature = "full", ts(skip))]
  pub my_user: Option<MyUserInfo>,
  pub all_languages: Vec<Language>,
  pub discussion_languages: Vec<LanguageId>,
  /// If the site has any taglines, a random one is included here for displaying
  #[cfg_attr(feature = "full", ts(optional))]
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
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct InstanceWithFederationState {
  #[serde(flatten)]
  pub instance: Instance,
  /// if federation to this instance is or was active, show state of outgoing federation to this
  /// instance
  #[cfg_attr(feature = "full", ts(optional))]
  pub federation_state: Option<ReadableFederationState>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Fetches a list of taglines.
pub struct ListTaglines {
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_cursor: Option<PaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_back: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response for taglines.
pub struct ListTaglinesResponse {
  pub taglines: Vec<Tagline>,
  /// the pagination cursor to use to fetch the next page
  #[cfg_attr(feature = "full", ts(optional))]
  pub next_page: Option<PaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub prev_page: Option<PaginationCursor>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response for a site.
pub struct SiteResponse {
  pub site_view: SiteView,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct TaglineResponse {
  pub tagline: Tagline,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Update a tagline
pub struct UpdateTagline {
  pub id: TaglineId,
  pub content: String,
}
