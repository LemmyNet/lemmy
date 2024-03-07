#[cfg(feature = "full")]
use crate::schema::local_site_url_blocklist;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = local_site_url_blocklist))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct LocalSiteUrlBlocklist {
  pub id: i32,
  pub url: String,
}

#[derive(Default, Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = local_site_url_blocklist))]
pub struct LocalSiteUrlBlocklistForm {
  pub url: String,
}
