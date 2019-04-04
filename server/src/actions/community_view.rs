extern crate diesel;
use diesel::*;
use diesel::result::Error;
use serde::{Deserialize, Serialize};

table! {
  community_view (id) {
    id -> Int4,
    name -> Varchar,
    title -> Varchar,
    description -> Nullable<Text>,
    category_id -> Int4,
    creator_id -> Int4,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
    creator_name -> Varchar,
    category_name -> Varchar,
    number_of_subscribers -> BigInt,
    number_of_posts -> BigInt,
    number_of_comments -> BigInt,
  }
}

table! {
  community_moderator_view (id) {
    id -> Int4,
    community_id -> Int4,
    user_id -> Int4,
    published -> Timestamp,
    user_name -> Varchar,
    community_name -> Varchar,
  }
}

table! {
  community_follower_view (id) {
    id -> Int4,
    community_id -> Int4,
    user_id -> Int4,
    published -> Timestamp,
    user_name -> Varchar,
    community_name -> Varchar,
  }
}

#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize,QueryableByName,Clone)]
#[table_name="community_view"]
pub struct CommunityView {
  pub id: i32,
  pub name: String,
  pub title: String,
  pub description: Option<String>,
  pub category_id: i32,
  pub creator_id: i32,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub creator_name: String,
  pub category_name: String,
  pub number_of_subscribers: i64,
  pub number_of_posts: i64,
  pub number_of_comments: i64
}

impl CommunityView {
  pub fn read(conn: &PgConnection, from_community_id: i32) -> Result<Self, Error> {
    use actions::community_view::community_view::dsl::*;
    community_view.find(from_community_id).first::<Self>(conn)
  }

  pub fn list_all(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use actions::community_view::community_view::dsl::*;
    community_view.load::<Self>(conn)
  }
}


#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize,QueryableByName,Clone)]
#[table_name="community_moderator_view"]
pub struct CommunityModeratorView {
  pub id: i32,
  pub community_id: i32,
  pub user_id: i32,
  pub published: chrono::NaiveDateTime,
  pub user_name : String,
  pub community_name: String,
}

impl CommunityModeratorView {
  pub fn for_community(conn: &PgConnection, from_community_id: i32) -> Result<Vec<Self>, Error> {
    use actions::community_view::community_moderator_view::dsl::*;
    community_moderator_view.filter(community_id.eq(from_community_id)).load::<Self>(conn)
  }

  pub fn for_user(conn: &PgConnection, from_user_id: i32) -> Result<Vec<Self>, Error> {
    use actions::community_view::community_moderator_view::dsl::*;
    community_moderator_view.filter(user_id.eq(from_user_id)).load::<Self>(conn)
  }
}

