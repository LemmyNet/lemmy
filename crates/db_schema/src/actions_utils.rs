use crate::{
  newtypes::PersonId,
  schema::{community, community_actions, instance_actions},
};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods};

#[diesel::dsl::auto_type]
pub fn community_actions_join(person_id: Option<PersonId>) -> _ {
  community_actions::table.on(
    community_actions::community_id
      .eq(community::id)
      .and(community_actions::person_id.nullable().eq(person_id)),
  )
}

#[diesel::dsl::auto_type]
pub fn instance_actions_join(person_id: Option<PersonId>) -> _ {
  instance_actions::table.on(
    instance_actions::instance_id
      .eq(community::instance_id)
      .and(instance_actions::person_id.nullable().eq(person_id)),
  )
}
