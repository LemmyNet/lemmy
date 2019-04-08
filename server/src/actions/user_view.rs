extern crate diesel;
use diesel::*;
use diesel::result::Error;
use serde::{Deserialize, Serialize};

table! {
  user_view (id) {
    id -> Int4,
    name -> Varchar,
    fedi_name -> Varchar,
    published -> Timestamp,
    number_of_posts -> BigInt,
    post_score -> BigInt,
    number_of_comments -> BigInt,
    comment_score -> BigInt,
  }
}

#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize,QueryableByName,Clone)]
#[table_name="user_view"]
pub struct UserView {
  pub id: i32,
  pub name: String,
  pub fedi_name: String,
  pub published: chrono::NaiveDateTime,
  pub number_of_posts: i64,
  pub post_score: i64,
  pub number_of_comments: i64,
  pub comment_score: i64,
}

impl UserView {
  pub fn read(conn: &PgConnection, from_user_id: i32) -> Result<Self, Error> {
    use actions::user_view::user_view::dsl::*;

    user_view.find(from_user_id)
    .first::<Self>(conn)
  }
}

