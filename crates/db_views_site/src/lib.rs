use lemmy_db_schema::source::{
  instance::Instance,
  local_site::LocalSite,
  local_site_rate_limit::LocalSiteRateLimit,
  site::Site,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  ts_rs::TS,
};

#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A site view.
pub struct SiteView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub site: Site,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_site: LocalSite,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub local_site_rate_limit: LocalSiteRateLimit,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance: Instance,
}
