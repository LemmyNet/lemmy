use crate::newtypes::{LocalSiteId, SiteId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::{
  CommentSortType,
  FederationMode,
  ListingType,
  PostListingMode,
  PostSortType,
  RegistrationMode,
};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::local_site;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = local_site))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::site::Site)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The local site.
pub struct LocalSite {
  pub id: LocalSiteId,
  pub site_id: SiteId,
  /// True if the site is set up.
  pub site_setup: bool,
  /// Whether only admins can create communities.
  pub community_creation_admin_only: bool,
  /// Whether emails are required.
  pub require_email_verification: bool,
  /// An optional registration application questionnaire in markdown.
  pub application_question: Option<String>,
  /// Whether the instance is private or public.
  pub private_instance: bool,
  /// The default front-end theme.
  pub default_theme: String,
  pub default_post_listing_type: ListingType,
  /// An optional legal disclaimer page.
  pub legal_information: Option<String>,
  /// Whether new applications email admins.
  pub application_email_admins: bool,
  /// An optional regex to filter words.
  pub slur_filter_regex: Option<String>,
  /// The max actor name length.
  pub actor_name_max_length: i32,
  /// Whether federation is enabled.
  pub federation_enabled: bool,
  /// Whether captcha is enabled.
  pub captcha_enabled: bool,
  /// The captcha difficulty.
  pub captcha_difficulty: String,
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  pub registration_mode: RegistrationMode,
  /// Whether to email admins on new reports.
  pub reports_email_admins: bool,
  /// Whether to sign outgoing Activitypub fetches with private key of local instance. Some
  /// Fediverse instances and platforms require this.
  pub federation_signed_fetch: bool,
  /// Default value for [LocalSite.post_listing_mode]
  pub default_post_listing_mode: PostListingMode,
  /// Default value for [LocalUser.post_sort_type]
  pub default_post_sort_type: PostSortType,
  /// Default value for [LocalUser.comment_sort_type]
  pub default_comment_sort_type: CommentSortType,
  /// Whether or not external auth methods can auto-register users.
  pub oauth_registration: bool,
  /// What kind of post upvotes your site allows.
  pub post_upvotes: FederationMode,
  /// What kind of post downvotes your site allows.
  pub post_downvotes: FederationMode,
  /// What kind of comment upvotes your site allows.
  pub comment_upvotes: FederationMode,
  /// What kind of comment downvotes your site allows.
  pub comment_downvotes: FederationMode,
  /// A default time range limit to apply to post sorts, in seconds.
  pub default_post_time_range_seconds: Option<i32>,
  /// Block NSFW content being created
  pub disallow_nsfw_content: bool,
  pub users: i64,
  pub posts: i64,
  pub comments: i64,
  pub communities: i64,
  /// The number of users with any activity in the last day.
  pub users_active_day: i64,
  /// The number of users with any activity in the last week.
  pub users_active_week: i64,
  /// The number of users with any activity in the last month.
  pub users_active_month: i64,
  /// The number of users with any activity in the last half year.
  pub users_active_half_year: i64,
  /// Dont send email notifications to users for new replies, mentions etc
  pub disable_email_notifications: bool,
}

#[derive(Clone, derive_new::new)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = local_site))]
pub struct LocalSiteInsertForm {
  pub site_id: SiteId,
  #[new(default)]
  pub site_setup: Option<bool>,
  #[new(default)]
  pub community_creation_admin_only: Option<bool>,
  #[new(default)]
  pub require_email_verification: Option<bool>,
  #[new(default)]
  pub application_question: Option<String>,
  #[new(default)]
  pub private_instance: Option<bool>,
  #[new(default)]
  pub default_theme: Option<String>,
  #[new(default)]
  pub default_post_listing_type: Option<ListingType>,
  #[new(default)]
  pub legal_information: Option<String>,
  #[new(default)]
  pub application_email_admins: Option<bool>,
  #[new(default)]
  pub slur_filter_regex: Option<String>,
  #[new(default)]
  pub actor_name_max_length: Option<i32>,
  #[new(default)]
  pub federation_enabled: Option<bool>,
  #[new(default)]
  pub captcha_enabled: Option<bool>,
  #[new(default)]
  pub captcha_difficulty: Option<String>,
  #[new(default)]
  pub registration_mode: Option<RegistrationMode>,
  #[new(default)]
  pub reports_email_admins: Option<bool>,
  #[new(default)]
  pub federation_signed_fetch: Option<bool>,
  #[new(default)]
  pub default_post_listing_mode: Option<PostListingMode>,
  #[new(default)]
  pub default_post_sort_type: Option<PostSortType>,
  #[new(default)]
  pub default_comment_sort_type: Option<CommentSortType>,
  #[new(default)]
  pub oauth_registration: Option<bool>,
  #[new(default)]
  pub post_upvotes: Option<FederationMode>,
  #[new(default)]
  pub post_downvotes: Option<FederationMode>,
  #[new(default)]
  pub comment_upvotes: Option<FederationMode>,
  #[new(default)]
  pub comment_downvotes: Option<FederationMode>,
  #[new(default)]
  pub default_post_time_range_seconds: Option<i32>,
  #[new(default)]
  pub disallow_nsfw_content: bool,
  #[new(default)]
  pub disable_email_notifications: bool,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = local_site))]
pub struct LocalSiteUpdateForm {
  pub site_setup: Option<bool>,
  pub community_creation_admin_only: Option<bool>,
  pub require_email_verification: Option<bool>,
  pub application_question: Option<Option<String>>,
  pub private_instance: Option<bool>,
  pub default_theme: Option<String>,
  pub default_post_listing_type: Option<ListingType>,
  pub legal_information: Option<Option<String>>,
  pub application_email_admins: Option<bool>,
  pub slur_filter_regex: Option<Option<String>>,
  pub actor_name_max_length: Option<i32>,
  pub federation_enabled: Option<bool>,
  pub captcha_enabled: Option<bool>,
  pub captcha_difficulty: Option<String>,
  pub registration_mode: Option<RegistrationMode>,
  pub reports_email_admins: Option<bool>,
  pub updated_at: Option<Option<DateTime<Utc>>>,
  pub federation_signed_fetch: Option<bool>,
  pub default_post_listing_mode: Option<PostListingMode>,
  pub default_post_sort_type: Option<PostSortType>,
  pub default_comment_sort_type: Option<CommentSortType>,
  pub oauth_registration: Option<bool>,
  pub post_upvotes: Option<FederationMode>,
  pub post_downvotes: Option<FederationMode>,
  pub comment_upvotes: Option<FederationMode>,
  pub comment_downvotes: Option<FederationMode>,
  pub default_post_time_range_seconds: Option<Option<i32>>,
  pub disallow_nsfw_content: Option<bool>,
  pub disable_email_notifications: Option<bool>,
}
