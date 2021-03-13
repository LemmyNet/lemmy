use crate::{schema::site, DbUrl};
use serde::Serialize;

#[derive(Queryable, Identifiable, PartialEq, Debug, Clone, Serialize)]
#[table_name = "site"]
pub struct Site {
  pub id: i32,
  pub name: String,
  pub description: Option<String>,
  pub creator_id: i32,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub enable_downvotes: bool,
  pub open_registration: bool,
  pub enable_nsfw: bool,
  pub icon: Option<DbUrl>,
  pub banner: Option<DbUrl>,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "site"]
pub struct SiteForm {
  pub name: String,
  pub description: Option<String>,
  pub creator_id: i32,
  pub updated: Option<chrono::NaiveDateTime>,
  pub enable_downvotes: bool,
  pub open_registration: bool,
  pub enable_nsfw: bool,
  // when you want to null out a column, you have to send Some(None)), since sending None means you just don't want to update that column.
  pub icon: Option<Option<DbUrl>>,
  pub banner: Option<Option<DbUrl>>,
}
