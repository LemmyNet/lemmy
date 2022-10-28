use crate::newtypes::{LocalSiteId, SiteId};
#[cfg(feature = "full")]
use crate::schema::local_site;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = local_site))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::site::Site)))]
pub struct LocalSite {
  pub id: LocalSiteId,
  pub site_id: SiteId,
  pub site_setup: bool,
  pub enable_downvotes: bool,
  pub open_registration: bool,
  pub enable_nsfw: bool,
  pub community_creation_admin_only: bool,
  pub require_email_verification: bool,
  pub require_application: bool,
  pub application_question: Option<String>,
  pub private_instance: bool,
  pub default_theme: String,
  pub default_post_listing_type: String,
  pub legal_information: Option<String>,
  pub hide_modlog_mod_names: bool,
  pub application_email_admins: bool,
  pub slur_filter_regex: Option<String>,
  pub actor_name_max_length: i32,
  pub federation_enabled: bool,
  pub federation_debug: bool,
  pub federation_strict_allowlist: bool,
  pub federation_http_fetch_retry_limit: i32,
  pub federation_worker_count: i32,
  pub captcha_enabled: bool,
  pub captcha_difficulty: String,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
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
  pub open_registration: Option<bool>,
  pub enable_nsfw: Option<bool>,
  pub community_creation_admin_only: Option<bool>,
  pub require_email_verification: Option<bool>,
  pub require_application: Option<bool>,
  pub application_question: Option<String>,
  pub private_instance: Option<bool>,
  pub default_theme: Option<String>,
  pub default_post_listing_type: Option<String>,
  pub legal_information: Option<String>,
  pub hide_modlog_mod_names: Option<bool>,
  pub application_email_admins: Option<bool>,
  pub slur_filter_regex: Option<String>,
  pub actor_name_max_length: Option<i32>,
  pub federation_enabled: Option<bool>,
  pub federation_debug: Option<bool>,
  pub federation_strict_allowlist: Option<bool>,
  pub federation_http_fetch_retry_limit: Option<i32>,
  pub federation_worker_count: Option<i32>,
  pub captcha_enabled: Option<bool>,
  pub captcha_difficulty: Option<String>,
}

#[derive(Clone, TypedBuilder)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = local_site))]
pub struct LocalSiteUpdateForm {
  pub site_setup: Option<bool>,
  pub enable_downvotes: Option<bool>,
  pub open_registration: Option<bool>,
  pub enable_nsfw: Option<bool>,
  pub community_creation_admin_only: Option<bool>,
  pub require_email_verification: Option<bool>,
  pub require_application: Option<bool>,
  pub application_question: Option<Option<String>>,
  pub private_instance: Option<bool>,
  pub default_theme: Option<String>,
  pub default_post_listing_type: Option<String>,
  pub legal_information: Option<Option<String>>,
  pub hide_modlog_mod_names: Option<bool>,
  pub application_email_admins: Option<bool>,
  pub slur_filter_regex: Option<Option<String>>,
  pub actor_name_max_length: Option<i32>,
  pub federation_enabled: Option<bool>,
  pub federation_debug: Option<bool>,
  pub federation_strict_allowlist: Option<bool>,
  pub federation_http_fetch_retry_limit: Option<i32>,
  pub federation_worker_count: Option<i32>,
  pub captcha_enabled: Option<bool>,
  pub captcha_difficulty: Option<String>,
  pub updated: Option<Option<chrono::NaiveDateTime>>,
}
