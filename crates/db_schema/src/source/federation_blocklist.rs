use crate::newtypes::InstanceId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
#[cfg(feature = "full")]
use {lemmy_db_schema_file::schema::federation_blocklist, ts_rs::TS};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(TS, Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::instance::Instance))
)]
#[cfg_attr(feature = "full", diesel(table_name = federation_blocklist))]
#[cfg_attr(feature = "full", diesel(primary_key(instance_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct FederationBlockList {
  pub instance_id: InstanceId,
  pub published_at: DateTime<Utc>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub updated_at: Option<DateTime<Utc>>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = federation_blocklist))]
pub struct FederationBlockListForm {
  pub instance_id: InstanceId,
  pub updated_at: Option<DateTime<Utc>>,
  pub expires_at: Option<DateTime<Utc>>,
}
