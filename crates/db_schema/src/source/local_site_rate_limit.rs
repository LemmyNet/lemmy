use crate::newtypes::LocalSiteId;
#[cfg(feature = "full")]
use crate::schema::local_site_rate_limit;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = local_site_rate_limit))]
#[cfg_attr(feature = "full", diesel(primary_key(local_site_id)))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::local_site::LocalSite))
)]
#[cfg_attr(feature = "full", ts(export))]
/// Rate limits for your site. Given in count / length of time.
pub struct LocalSiteRateLimit {
  pub local_site_id: LocalSiteId,
  pub message: i32,
  pub message_per_second: i32,
  pub post: i32,
  pub post_per_second: i32,
  pub register: i32,
  pub register_per_second: i32,
  pub image: i32,
  pub image_per_second: i32,
  pub comment: i32,
  pub comment_per_second: i32,
  pub search: i32,
  pub search_per_second: i32,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
  pub import_user_settings: i32,
  pub import_user_settings_per_second: i32,
}

#[derive(Clone, TypedBuilder)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = local_site_rate_limit))]
pub struct LocalSiteRateLimitInsertForm {
  #[builder(!default)]
  pub local_site_id: LocalSiteId,
  pub message: Option<i32>,
  pub message_per_second: Option<i32>,
  pub post: Option<i32>,
  pub post_per_second: Option<i32>,
  pub register: Option<i32>,
  pub register_per_second: Option<i32>,
  pub image: Option<i32>,
  pub image_per_second: Option<i32>,
  pub comment: Option<i32>,
  pub comment_per_second: Option<i32>,
  pub search: Option<i32>,
  pub search_per_second: Option<i32>,
  pub import_user_settings: Option<i32>,
  pub import_user_settings_per_second: Option<i32>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = local_site_rate_limit))]
pub struct LocalSiteRateLimitUpdateForm {
  pub message: Option<i32>,
  pub message_per_second: Option<i32>,
  pub post: Option<i32>,
  pub post_per_second: Option<i32>,
  pub register: Option<i32>,
  pub register_per_second: Option<i32>,
  pub image: Option<i32>,
  pub image_per_second: Option<i32>,
  pub comment: Option<i32>,
  pub comment_per_second: Option<i32>,
  pub search: Option<i32>,
  pub search_per_second: Option<i32>,
  pub import_user_settings: Option<i32>,
  pub import_user_settings_per_second: Option<i32>,
  pub updated: Option<Option<DateTime<Utc>>>,
}
