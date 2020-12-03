use crate::schema::site_aggregates;
use diesel::{result::Error, *};
use serde::Serialize;

#[derive(Queryable, Associations, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "site_aggregates"]
pub struct SiteAggregates {
  pub id: i32,
  pub users: i64,
  pub posts: i64,
  pub comments: i64,
  pub communities: i64,
}

impl SiteAggregates {
  pub fn read(conn: &PgConnection) -> Result<Self, Error> {
    site_aggregates::table.first::<Self>(conn)
  }
}
