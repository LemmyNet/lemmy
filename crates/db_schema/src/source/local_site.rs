#[cfg(feature = "full")]
use crate::schema::local_site;
use crate::{
  newtypes::{LocalSiteId, SiteId},
  ListingType,
  RegistrationMode,
  SortType,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = local_site))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::site::Site)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// The local site.
pub struct LocalSite {
  pub id: LocalSiteId,
  pub site_id: SiteId,
  /// True if the site is set up.
  pub site_setup: bool,
  /// Whether downvotes are enabled.
  pub enable_downvotes: bool,
  /// Whether NSFW is enabled.
  pub enable_nsfw: bool,
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
  /// Whether to hide mod names on the modlog.
  pub hide_modlog_mod_names: bool,
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
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
  pub registration_mode: RegistrationMode,
  /// Whether to email admins on new reports.
  pub reports_email_admins: bool,
  /// Whether to sign outgoing Activitypub fetches with private key of local instance. Some
  /// Fediverse instances and platforms require this.
  pub federation_signed_fetch: bool,
  pub default_sort_type: SortType,
}

#[derive(Clone, TypedBuilder)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = local_site))]
pub struct LocalSiteInsertForm {
  #[builder(!default)]
  pub site_id: SiteId,
  pub site_setup: Option<bool>,
  pub enable_downvotes: Option<bool>,
  pub enable_nsfw: Option<bool>,
  pub community_creation_admin_only: Option<bool>,
  pub require_email_verification: Option<bool>,
  pub application_question: Option<String>,
  pub private_instance: Option<bool>,
  pub default_theme: Option<String>,
  pub default_post_listing_type: Option<ListingType>,
  pub legal_information: Option<String>,
  pub hide_modlog_mod_names: Option<bool>,
  pub application_email_admins: Option<bool>,
  pub slur_filter_regex: Option<String>,
  pub actor_name_max_length: Option<i32>,
  pub federation_enabled: Option<bool>,
  pub captcha_enabled: Option<bool>,
  pub captcha_difficulty: Option<String>,
  pub registration_mode: Option<RegistrationMode>,
  pub reports_email_admins: Option<bool>,
  pub federation_signed_fetch: Option<bool>,
  pub default_sort_type: Option<SortType>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = local_site))]
pub struct LocalSiteUpdateForm {
  pub site_setup: Option<bool>,
  pub enable_downvotes: Option<bool>,
  pub enable_nsfw: Option<bool>,
  pub community_creation_admin_only: Option<bool>,
  pub require_email_verification: Option<bool>,
  pub application_question: Option<Option<String>>,
  pub private_instance: Option<bool>,
  pub default_theme: Option<String>,
  pub default_post_listing_type: Option<ListingType>,
  pub legal_information: Option<Option<String>>,
  pub hide_modlog_mod_names: Option<bool>,
  pub application_email_admins: Option<bool>,
  pub slur_filter_regex: Option<Option<String>>,
  pub actor_name_max_length: Option<i32>,
  pub federation_enabled: Option<bool>,
  pub captcha_enabled: Option<bool>,
  pub captcha_difficulty: Option<String>,
  pub registration_mode: Option<RegistrationMode>,
  pub reports_email_admins: Option<bool>,
  pub updated: Option<Option<DateTime<Utc>>>,
  pub federation_signed_fetch: Option<bool>,
  pub default_sort_type: Option<SortType>,
}
