use diesel::{result::Error, *};
use lemmy_db_queries::{aggregates::site_aggregates::SiteAggregates, ToSafe};
use lemmy_db_schema::{
  schema::{site, site_aggregates, user_},
  source::{
    site::Site,
    user::{UserSafe, User_},
  },
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SiteView {
  pub site: Site,
  pub creator: UserSafe,
  pub counts: SiteAggregates,
}

impl SiteView {
  pub fn read(conn: &PgConnection) -> Result<Self, Error> {
    let (site, creator, counts) = site::table
      .inner_join(user_::table)
      .inner_join(site_aggregates::table)
      .select((
        site::all_columns,
        User_::safe_columns_tuple(),
        site_aggregates::all_columns,
      ))
      .first::<(Site, UserSafe, SiteAggregates)>(conn)?;

    Ok(SiteView {
      site,
      creator,
      counts,
    })
  }
}
