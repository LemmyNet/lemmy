use crate::{newtypes::InstanceId, schema::blocklist};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::instance::Instance))
)]
#[cfg_attr(feature = "full", diesel(table_name = blocklist))]
pub struct BlockList {
  pub id: i32,
  pub instance_id: InstanceId,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = blocklist))]
pub struct BlockListForm {
  pub instance_id: InstanceId,
  pub updated: Option<chrono::NaiveDateTime>,
}
