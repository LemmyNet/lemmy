use lemmy_db_schema::newtypes::LanguageId;
use lemmy_db_schema_file::enums::{
  CommentSortType, FederationMode, ListingType, PostListingMode, PostSortType, RegistrationMode,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

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
  /// Whether to hide moderator names from the modlog.
  #[cfg_attr(feature = "full", ts(optional))]
  pub hide_modlog_mod_names: Option<bool>,
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
