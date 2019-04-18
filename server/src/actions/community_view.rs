extern crate diesel;
use diesel::*;
use diesel::result::Error;
use serde::{Deserialize, Serialize};
use {SortType, limit_and_offset};

table! {
  community_view (id) {
    id -> Int4,
    name -> Varchar,
    title -> Varchar,
    description -> Nullable<Text>,
    category_id -> Int4,
    creator_id -> Int4,
    removed -> Nullable<Bool>,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
    creator_name -> Varchar,
    category_name -> Varchar,
    number_of_subscribers -> BigInt,
    number_of_posts -> BigInt,
    number_of_comments -> BigInt,
    user_id -> Nullable<Int4>,
    subscribed -> Nullable<Bool>,
    am_mod -> Nullable<Bool>,
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

table! {
  community_user_ban_view (id) {
    id -> Int4,
    community_id -> Int4,
    user_id -> Int4,
    published -> Timestamp,
    user_name -> Varchar,
    community_name -> Varchar,
  }
}

table! {
    site_view (id) {
      id -> Int4,
      name -> Varchar,
      description -> Nullable<Text>,
      creator_id -> Int4,
      published -> Timestamp,
      updated -> Nullable<Timestamp>,
      creator_name -> Varchar,
      number_of_users -> BigInt,
      number_of_posts -> BigInt,
      number_of_comments -> BigInt,
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
  pub removed: Option<bool>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub creator_name: String,
  pub category_name: String,
  pub number_of_subscribers: i64,
  pub number_of_posts: i64,
  pub number_of_comments: i64,
  pub user_id: Option<i32>,
  pub subscribed: Option<bool>,
  pub am_mod: Option<bool>,
}

impl CommunityView {
  pub fn read(conn: &PgConnection, from_community_id: i32, from_user_id: Option<i32>) -> Result<Self, Error> {
    use actions::community_view::community_view::dsl::*;

    let mut query = community_view.into_boxed();

    query = query.filter(id.eq(from_community_id));

    // The view lets you pass a null user_id, if you're not logged in
    if let Some(from_user_id) = from_user_id {
      query = query.filter(user_id.eq(from_user_id));
    } else {
      query = query.filter(user_id.is_null());
    };

    query.first::<Self>(conn)
  }

  pub fn list(conn: &PgConnection, 
              from_user_id: Option<i32>, 
              sort: SortType, 
              page: Option<i64>,
              limit: Option<i64>,
              ) -> Result<Vec<Self>, Error> {
    use actions::community_view::community_view::dsl::*;
    let mut query = community_view.into_boxed();

    let (limit, offset) = limit_and_offset(page, limit);

    // The view lets you pass a null user_id, if you're not logged in
    match sort {
      SortType::New => query = query.order_by(published.desc()).filter(user_id.is_null()),
      SortType::TopAll => {
        match from_user_id {
          Some(from_user_id) => query = query.filter(user_id.eq(from_user_id)).order_by((subscribed.asc(), number_of_subscribers.desc())),
          None => query = query.order_by(number_of_subscribers.desc()).filter(user_id.is_null())
        }
      }
      _ => ()
    };

    query
      .limit(limit)
      .offset(offset)
      .filter(removed.eq(false))
      .load::<Self>(conn) 
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

#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize,QueryableByName,Clone)]
#[table_name="community_follower_view"]
pub struct CommunityFollowerView {
  pub id: i32,
  pub community_id: i32,
  pub user_id: i32,
  pub published: chrono::NaiveDateTime,
  pub user_name : String,
  pub community_name: String,
}

impl CommunityFollowerView {
  pub fn for_community(conn: &PgConnection, from_community_id: i32) -> Result<Vec<Self>, Error> {
    use actions::community_view::community_follower_view::dsl::*;
    community_follower_view.filter(community_id.eq(from_community_id)).load::<Self>(conn)
  }

  pub fn for_user(conn: &PgConnection, from_user_id: i32) -> Result<Vec<Self>, Error> {
    use actions::community_view::community_follower_view::dsl::*;
    community_follower_view.filter(user_id.eq(from_user_id)).load::<Self>(conn)
  }
}


#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize,QueryableByName,Clone)]
#[table_name="community_user_ban_view"]
pub struct CommunityUserBanView {
  pub id: i32,
  pub community_id: i32,
  pub user_id: i32,
  pub published: chrono::NaiveDateTime,
  pub user_name : String,
  pub community_name: String,
}

impl CommunityUserBanView {
  pub fn for_community(conn: &PgConnection, from_community_id: i32) -> Result<Vec<Self>, Error> {
    use actions::community_view::community_user_ban_view::dsl::*;
    community_user_ban_view.filter(community_id.eq(from_community_id)).load::<Self>(conn)
  }

  pub fn for_user(conn: &PgConnection, from_user_id: i32) -> Result<Vec<Self>, Error> {
    use actions::community_view::community_user_ban_view::dsl::*;
    community_user_ban_view.filter(user_id.eq(from_user_id)).load::<Self>(conn)
  }

  pub fn get(conn: &PgConnection, from_user_id: i32, from_community_id: i32) -> Result<Self, Error> {
    use actions::community_view::community_user_ban_view::dsl::*;
    community_user_ban_view
      .filter(user_id.eq(from_user_id))
      .filter(community_id.eq(from_community_id))
      .first::<Self>(conn)
  }
}


#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize,QueryableByName,Clone)]
#[table_name="site_view"]
pub struct SiteView {
  pub id: i32,
  pub name: String,
  pub description: Option<String>,
  pub creator_id: i32,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub creator_name: String,
  pub number_of_users: i64,
  pub number_of_posts: i64,
  pub number_of_comments: i64,
}

impl SiteView {
  pub fn read(conn: &PgConnection) -> Result<Self, Error> {
    use actions::community_view::site_view::dsl::*;
    site_view.first::<Self>(conn)
  }
}
