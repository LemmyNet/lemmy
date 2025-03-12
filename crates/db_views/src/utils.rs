use diesel::{
  helper_types::{Eq, NotEq, Or},
  BoolExpressionMethods,
  ExpressionMethods,
};
use lemmy_db_schema::{
  schema::{community, community_actions, instance_actions, person_actions},
  source::community::CommunityFollowerState,
  CommunityVisibility,
};

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

type IsSubscribedType =
  Eq<lemmy_db_schema::schema::community_actions::follow_state, Option<CommunityFollowerState>>;

pub(crate) fn filter_is_subscribed() -> IsSubscribedType {
  community_actions::follow_state.eq(Some(CommunityFollowerState::Accepted))
}

type IsNotHiddenType = NotEq<lemmy_db_schema::schema::community::visibility, CommunityVisibility>;

pub(crate) fn filter_not_hidden_or_is_subscribed() -> Or<IsNotHiddenType, IsSubscribedType> {
  let not_hidden = community::visibility.ne(CommunityVisibility::Unlisted);
  not_hidden.or(filter_is_subscribed())
}
