use crate::schema::comment_aggregates;
use diesel::{result::Error, *};
use serde::Serialize;

#[derive(Queryable, Associations, Identifiable, PartialEq, Debug, Serialize, Clone)]
#[table_name = "comment_aggregates"]
pub struct CommentAggregates {
  pub id: i32,
  pub comment_id: i32,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
}

impl CommentAggregates {
  pub fn read(conn: &PgConnection, comment_id: i32) -> Result<Self, Error> {
    comment_aggregates::table
      .filter(comment_aggregates::comment_id.eq(comment_id))
      .first::<Self>(conn)
  }
}

// TODO add tests here
