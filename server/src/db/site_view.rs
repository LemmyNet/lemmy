use super::*;

table! {
  site_view (id) {
    id -> Int4,
    name -> Varchar,
    description -> Nullable<Text>,
    creator_id -> Int4,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
    enable_downvotes -> Bool,
    open_registration -> Bool,
    enable_nsfw -> Bool,
    creator_name -> Varchar,
    creator_avatar -> Nullable<Text>,
    number_of_users -> BigInt,
    number_of_posts -> BigInt,
    number_of_comments -> BigInt,
    number_of_communities -> BigInt,
  }
}

#[derive(
  Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize, QueryableByName, Clone,
)]
#[table_name = "site_view"]
pub struct SiteView {
  pub id: i32,
  pub name: String,
  pub description: Option<String>,
  pub creator_id: i32,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub enable_downvotes: bool,
  pub open_registration: bool,
  pub enable_nsfw: bool,
  pub creator_name: String,
  pub creator_avatar: Option<String>,
  pub number_of_users: i64,
  pub number_of_posts: i64,
  pub number_of_comments: i64,
  pub number_of_communities: i64,
}

impl SiteView {
  pub fn read(conn: &PgConnection) -> Result<Self, Error> {
    use super::site_view::site_view::dsl::*;
    site_view.first::<Self>(conn)
  }
}
