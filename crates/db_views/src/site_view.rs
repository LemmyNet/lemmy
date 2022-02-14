use diesel::{result::Error, *};
use lemmy_db_schema::{
  aggregates::site_aggregates::SiteAggregates,
  schema::{site, site_aggregates},
  source::site::Site,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteView {
  pub site: Site,
  pub counts: SiteAggregates,
}

impl SiteView {
  pub fn read(conn: &PgConnection) -> Result<Self, Error> {
    let (mut site, counts) = site::table
      .inner_join(site_aggregates::table)
      .select((site::all_columns, site_aggregates::all_columns))
      .first::<(Site, SiteAggregates)>(conn)?;

    site.private_key = None;
    Ok(SiteView { site, counts })
  }
}
