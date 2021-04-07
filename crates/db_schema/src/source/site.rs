use crate::{schema::site, DbUrl, PersonId};
use serde::Serialize;

#[derive(Queryable, Identifiable, PartialEq, Debug, Clone, Serialize)]
#[table_name = "site"]
pub struct Site {
  pub id: i32,
  pub name: String,
  pub sidebar: Option<String>,
  pub creator_id: PersonId,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub enable_downvotes: bool,
  pub open_registration: bool,
  pub enable_nsfw: bool,
  pub icon: Option<DbUrl>,
  pub banner: Option<DbUrl>,
  pub description: Option<String>,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "site"]
pub struct SiteForm {
  pub name: String,
  pub sidebar: Option<Option<String>>,
  pub creator_id: PersonId,
  pub updated: Option<chrono::NaiveDateTime>,
  pub enable_downvotes: bool,
  pub open_registration: bool,
  pub enable_nsfw: bool,
  // when you want to null out a column, you have to send Some(None)), since sending None means you just don't want to update that column.
  pub icon: Option<Option<DbUrl>>,
  pub banner: Option<Option<DbUrl>>,
  pub description: Option<Option<String>>,
}
