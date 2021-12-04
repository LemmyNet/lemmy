use crate::{
  newtypes::{DbUrl, PersonId},
  schema::site,
};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Identifiable, PartialEq, Debug, Clone, Serialize, Deserialize)]
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
  pub community_creation_admin_only: bool,
  pub require_email_verification: bool,
  pub require_application: bool,
  pub application_question: Option<String>,
}

#[derive(Insertable, AsChangeset, Default)]
#[table_name = "site"]
pub struct SiteForm {
  pub name: String,
  pub creator_id: PersonId,
  pub sidebar: Option<Option<String>>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub enable_downvotes: Option<bool>,
  pub open_registration: Option<bool>,
  pub enable_nsfw: Option<bool>,
  // when you want to null out a column, you have to send Some(None)), since sending None means you just don't want to update that column.
  pub icon: Option<Option<DbUrl>>,
  pub banner: Option<Option<DbUrl>>,
  pub description: Option<Option<String>>,
  pub community_creation_admin_only: Option<bool>,
  pub require_email_verification: Option<bool>,
  pub require_application: Option<bool>,
  pub application_question: Option<Option<String>>,
}
