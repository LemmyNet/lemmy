#[cfg(feature = "full")]
use diesel::Queryable;
use lemmy_db_schema::source::{community::Community, person::Person};
use lemmy_db_schema_file::enums::CommunityFollowerState;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct PendingFollowerView {
  pub person: Person,
  pub community: Community,
  pub is_new_instance: bool,
  pub follow_state: Option<CommunityFollowerState>,
}
