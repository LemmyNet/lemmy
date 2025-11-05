use crate::aliases::my_instance_persons_actions;
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
  helper_types::{Eq, NotEq},
};
use lemmy_db_schema_file::{
  enums::{CommunityFollowerState, CommunityVisibility},
  schema::{
    community,
    community_actions,
    instance_actions,
    local_site,
    multi_community,
    multi_community_entry,
    person_actions,
  },
};

/// Hide all content from blocked communities and persons. Content from blocked instances is also
/// hidden, unless the user followed the community explicitly.
#[diesel::dsl::auto_type]
pub fn filter_blocked() -> _ {
  instance_actions::blocked_communities_at
    .is_null()
    .or(community_actions::followed_at.is_not_null())
    .and(community_actions::blocked_at.is_null())
    .and(person_actions::blocked_at.is_null())
    .and(
      my_instance_persons_actions
        .field(instance_actions::blocked_persons_at)
        .is_null(),
    )
}

type IsSubscribedType =
  Eq<lemmy_db_schema_file::schema::community_actions::follow_state, Option<CommunityFollowerState>>;

pub fn filter_is_subscribed() -> IsSubscribedType {
  community_actions::follow_state.eq(Some(CommunityFollowerState::Accepted))
}

type IsNotUnlistedType =
  NotEq<lemmy_db_schema_file::schema::community::visibility, CommunityVisibility>;

#[diesel::dsl::auto_type]
pub fn filter_not_unlisted_or_is_subscribed() -> _ {
  let not_unlisted: IsNotUnlistedType = community::visibility.ne(CommunityVisibility::Unlisted);
  let is_subscribed: IsSubscribedType = filter_is_subscribed();
  not_unlisted.or(is_subscribed)
}

#[diesel::dsl::auto_type]
pub fn filter_suggested_communities() -> _ {
  community::id.eq_any(
    local_site::table
      .left_join(multi_community::table.inner_join(multi_community_entry::table))
      .filter(multi_community_entry::community_id.is_not_null())
      .select(multi_community_entry::community_id.assume_not_null()),
  )
}
