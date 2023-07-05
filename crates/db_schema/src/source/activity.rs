use crate::{newtypes::DbUrl, schema::activity};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::fmt::Debug;

#[derive(PartialEq, Eq, Debug, Queryable, Identifiable)]
#[diesel(table_name = activity)]
pub struct Activity {
  pub id: i32,
  pub data: Value,
  pub local: bool,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
  pub ap_id: DbUrl,
  pub sensitive: bool,
}

#[derive(Insertable)]
#[diesel(table_name = activity)]
pub struct ActivityInsertForm {
  pub data: Value,
  pub local: Option<bool>,
  pub updated: Option<DateTime<Utc>>,
  pub ap_id: DbUrl,
  pub sensitive: Option<bool>,
}

#[derive(AsChangeset)]
#[diesel(table_name = activity)]
pub struct ActivityUpdateForm {
  pub data: Option<Value>,
  pub local: Option<bool>,
  pub updated: Option<Option<DateTime<Utc>>>,
  pub sensitive: Option<bool>,
}
