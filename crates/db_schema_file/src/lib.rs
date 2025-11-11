use core::default::Default;
#[cfg(feature = "full")]
use diesel_derive_newtype::DieselNewType;
use serde::{Deserialize, Serialize};

pub mod enums;
#[cfg(feature = "full")]
pub mod joins;
#[cfg(feature = "full")]
pub mod schema;
#[cfg(feature = "full")]
pub mod table_impls;

#[cfg(feature = "full")]
pub mod aliases {
  use crate::schema::{community_actions, instance_actions, local_user, person};
  diesel::alias!(
    community_actions as creator_community_actions: CreatorCommunityActions,
    instance_actions as creator_home_instance_actions: CreatorHomeInstanceActions,
    instance_actions as creator_community_instance_actions: CreatorCommunityInstanceActions,
    instance_actions as creator_local_instance_actions: CreatorLocalInstanceActions,
    instance_actions as my_instance_persons_actions: MyInstancePersonsActions,
    local_user as creator_local_user: CreatorLocalUser,
    person as person1: Person1,
    person as person2: Person2,
  );
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The person id.
pub struct PersonId(pub i32);

#[derive(
  Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Default, Ord, PartialOrd,
)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The instance id.
pub struct InstanceId(pub i32);

impl InstanceId {
  pub fn inner(self) -> i32 {
    self.0
  }
}
