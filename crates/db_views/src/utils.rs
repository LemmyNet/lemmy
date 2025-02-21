use diesel::{BoolExpressionMethods, ExpressionMethods};
use lemmy_db_schema::schema::{community_actions, instance_actions, person_actions};

/// Hide all content from blocked communities and persons. Content from blocked instances is also
/// hidden, unless the user followed the community explicitly.
#[diesel::dsl::auto_type]
pub(crate) fn filter_blocked() -> _ {
  instance_actions::blocked
    .is_null()
    .or(community_actions::followed.is_not_null())
    .and(community_actions::blocked.is_null())
    .and(person_actions::blocked.is_null())
}
