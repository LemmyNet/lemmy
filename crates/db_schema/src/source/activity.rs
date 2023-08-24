use crate::{newtypes::DbUrl, schema::sent_activity};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::fmt::Debug;

#[derive(PartialEq, Eq, Debug, Queryable)]
#[diesel(table_name = sent_activity)]
pub struct SentActivity {
  pub id: i64,
  pub ap_id: DbUrl,
  pub data: Value,
  pub sensitive: bool,
  pub published: DateTime<Utc>,
}
#[derive(Insertable)]
#[diesel(table_name = sent_activity)]
pub struct SentActivityForm {
  pub ap_id: DbUrl,
  pub data: Value,
  pub sensitive: bool,
}

#[derive(PartialEq, Eq, Debug, Queryable)]
#[diesel(table_name = received_activity)]
pub struct ReceivedActivity {
  pub id: i64,
  pub ap_id: DbUrl,
  pub published: DateTime<Utc>,
}
