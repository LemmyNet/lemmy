use crate::structs::SiteView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  aggregates::structs::SiteAggregates,
  schema::{local_site, site, site_aggregates},
  source::{local_site::LocalSite, site::Site},
};

impl SiteView {
  pub fn read_local(conn: &mut PgConnection) -> Result<Self, Error> {
    let (mut site, local_site, counts) = site::table
      .inner_join(local_site::table)
      .inner_join(site_aggregates::table)
      .select((
        site::all_columns,
        local_site::all_columns,
        site_aggregates::all_columns,
      ))
      .first::<(Site, LocalSite, SiteAggregates)>(conn)?;

    site.private_key = None;
    Ok(SiteView {
      site,
      local_site,
      counts,
    })
  }
}
