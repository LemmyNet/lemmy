use diesel::{result::Error, *};
use lemmy_db_queries::{aggregates::site_aggregates::SiteAggregates, ToSafe};
use lemmy_db_schema::{
  schema::{person, site, site_aggregates},
  source::{
    person::{Person, PersonSafe},
    site::Site,
  },
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SiteView {
  pub site: Site,
  pub creator: PersonSafe,
  pub counts: SiteAggregates,
}

impl SiteView {
  pub fn read(conn: &PgConnection) -> Result<Self, Error> {
    let (site, creator, counts) = site::table
      .inner_join(person::table)
      .inner_join(site_aggregates::table)
      .select((
        site::all_columns,
        Person::safe_columns_tuple(),
        site_aggregates::all_columns,
      ))
      .first::<(Site, PersonSafe, SiteAggregates)>(conn)?;

    Ok(SiteView {
      site,
      creator,
      counts,
    })
  }
}
