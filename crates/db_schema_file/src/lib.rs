pub mod enums;
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