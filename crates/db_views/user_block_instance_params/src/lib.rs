use lemmy_db_schema::newtypes::InstanceId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Block an instance as user
pub struct UserBlockInstanceParams {
  pub instance_id: InstanceId,
  pub block: bool,
}
