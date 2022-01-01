use crate::{
     newtypes::{ PersonId},
    schema::{blacklist_community},
  };
  use serde::{Deserialize, Serialize};

  #[derive(
    Clone, Queryable, Associations, Identifiable, PartialEq, Debug, Serialize, Deserialize,
  )]
#[table_name = "blacklist_community"]
pub struct BlackList {
  pub id: i32,
  pub community_id: i32,
  pub reason: Option<String>,
  pub published: chrono::NaiveDateTime,
  pub creator_id: PersonId,
}

#[derive(Insertable, AsChangeset, Clone, Default)]
#[table_name = "blacklist_community"]
pub struct BlackListForm {
  pub reason: Option<String>,
  pub published: Option<chrono::NaiveDateTime>,
  pub creator_id: PersonId,
  pub community_id: i32
}