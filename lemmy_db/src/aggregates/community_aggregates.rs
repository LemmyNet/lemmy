use crate::schema::community_aggregates;
use diesel::{result::Error, *};
use serde::Serialize;

#[derive(Queryable, Associations, Identifiable, PartialEq, Debug, Serialize, Clone)]
#[table_name = "community_aggregates"]
pub struct CommunityAggregates {
  pub id: i32,
  pub community_id: i32,
  pub subscribers: i64,
  pub posts: i64,
  pub counts: i64,
}

impl CommunityAggregates {
  pub fn read(conn: &PgConnection, id: i32) -> Result<Self, Error> {
    community_aggregates::table.find(id).first::<Self>(conn)
  }
}

// TODO add unit tests, to make sure triggers are working
