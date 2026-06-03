use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  dsl::IsNotNull,
  helper_types::{Eq, NotEq, Or},
};
use lemmy_db_schema_file::{
  aliases::my_instance_persons_actions,
  enums::{CommunityFollowerState, CommunityVisibility},
  schema::{community, community_actions, instance_actions, local_user, person_actions},
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

type IsSubscribedType = Eq<community_actions::follow_state, Option<CommunityFollowerState>>;

pub fn filter_is_subscribed() -> IsSubscribedType {
  community_actions::follow_state.eq(Some(CommunityFollowerState::Accepted))
}

type CommunityVisibilityType = NotEq<community::visibility, CommunityVisibility>;

type CommunityVisibilityNotUnlistedOrSubscribedType = Or<CommunityVisibilityType, IsSubscribedType>;

/// Show only listed or followed communities
pub fn filter_unlisted_or_followed() -> CommunityVisibilityNotUnlistedOrSubscribedType {
  community::visibility
    .ne(CommunityVisibility::Unlisted)
    .or(filter_is_subscribed())
}

type CommunityVisibilityOrSubscribedType = Or<
  Or<
    Or<CommunityVisibilityType, IsSubscribedType>,
    IsNotNull<community_actions::became_moderator_at>,
  >,
  local_user::admin,
>;

/// Show only non-private or followed communities
pub fn filter_private_or_followed() -> CommunityVisibilityOrSubscribedType {
  community::visibility
    .ne(CommunityVisibility::Private)
    .or(filter_is_subscribed())
    .or(community_actions::became_moderator_at.is_not_null())
    .or(local_user::admin)
}
