use crate::schema::user_aggregates;
use diesel::{result::Error, *};
use serde::Serialize;

#[derive(Queryable, Associations, Identifiable, PartialEq, Debug, Serialize, Clone)]
#[table_name = "user_aggregates"]
pub struct UserAggregates {
  pub id: i32,
  pub user_id: i32,
  pub post_count: i64,
  pub post_score: i64,
  pub comment_count: i64,
  pub comment_score: i64,
}

impl UserAggregates {
  pub fn read(conn: &PgConnection, id: i32) -> Result<Self, Error> {
    user_aggregates::table.find(id).first::<Self>(conn)
  }
}

// TODO add unit tests, to make sure triggers are working
