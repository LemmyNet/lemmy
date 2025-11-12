use crate::{
  InstanceId,
  PersonId,
  aliases::{
    creator_community_actions,
    creator_community_instance_actions,
    creator_home_instance_actions,
    creator_local_instance_actions,
    creator_local_user,
    my_instance_persons_actions,
  },
  schema::{
    comment,
    comment_actions,
    community,
    community_actions,
    image_details,
    instance_actions,
    local_user,
    multi_community,
    multi_community_follow,
    person,
    person_actions,
    post,
    post_actions,
  },
};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods};

#[diesel::dsl::auto_type]
pub fn creator_local_user_admin_join() -> _ {
  creator_local_user.on(
    person::id
      .eq(creator_local_user.field(local_user::person_id))
      .and(creator_local_user.field(local_user::admin).eq(true)),
  )
}

#[diesel::dsl::auto_type]
pub fn community_join() -> _ {
  community::table.on(post::community_id.eq(community::id))
}
#[diesel::dsl::auto_type]
pub fn creator_home_instance_actions_join() -> _ {
  creator_home_instance_actions.on(
    creator_home_instance_actions
      .field(instance_actions::instance_id)
      .eq(person::instance_id)
      .and(
        creator_home_instance_actions
          .field(instance_actions::person_id)
          .eq(person::id),
      ),
  )
}
#[diesel::dsl::auto_type]
pub fn creator_community_instance_actions_join() -> _ {
  creator_community_instance_actions.on(
    creator_home_instance_actions
      .field(instance_actions::instance_id)
      .eq(community::instance_id)
      .and(
        creator_community_instance_actions
          .field(instance_actions::person_id)
          .eq(person::id),
      ),
  )
}

/// join with instance actions for local instance
///
/// Requires annotation for return type, see https://docs.diesel.rs/2.2.x/diesel/dsl/attr.auto_type.html#annotating-types
#[diesel::dsl::auto_type]
pub fn creator_local_instance_actions_join(local_instance_id: InstanceId) -> _ {
  creator_local_instance_actions.on(
    creator_local_instance_actions
      .field(instance_actions::instance_id)
      .eq(local_instance_id)
      .and(
        creator_local_instance_actions
          .field(instance_actions::person_id)
          .eq(person::id),
      ),
  )
}

/// Your instance actions for the community's instance.
#[diesel::dsl::auto_type]
pub fn my_instance_communities_actions_join(my_person_id: Option<PersonId>) -> _ {
  instance_actions::table.on(
    instance_actions::instance_id
      .eq(community::instance_id)
      .and(instance_actions::person_id.nullable().eq(my_person_id)),
  )
}

/// Your instance actions for the person's instance.
#[diesel::dsl::auto_type]
pub fn my_instance_persons_actions_join(my_person_id: Option<PersonId>) -> _ {
  instance_actions::table.on(
    instance_actions::instance_id
      .eq(person::instance_id)
      .and(instance_actions::person_id.nullable().eq(my_person_id)),
  )
}

/// Your instance actions for the person's instance.
/// A dupe of the above function, but aliased
#[diesel::dsl::auto_type]
pub fn my_instance_persons_actions_join_1(my_person_id: Option<PersonId>) -> _ {
  my_instance_persons_actions.on(
    my_instance_persons_actions
      .field(instance_actions::instance_id)
      .eq(person::instance_id)
      .and(
        my_instance_persons_actions
          .field(instance_actions::person_id)
          .nullable()
          .eq(my_person_id),
      ),
  )
}

#[diesel::dsl::auto_type]
pub fn image_details_join() -> _ {
  image_details::table.on(post::thumbnail_url.eq(image_details::link.nullable()))
}

#[diesel::dsl::auto_type]
pub fn my_community_actions_join(my_person_id: Option<PersonId>) -> _ {
  community_actions::table.on(
    community_actions::community_id
      .eq(community::id)
      .and(community_actions::person_id.nullable().eq(my_person_id)),
  )
}

#[diesel::dsl::auto_type]
pub fn my_post_actions_join(my_person_id: Option<PersonId>) -> _ {
  post_actions::table.on(
    post_actions::post_id
      .eq(post::id)
      .and(post_actions::person_id.nullable().eq(my_person_id)),
  )
}

#[diesel::dsl::auto_type]
pub fn my_comment_actions_join(my_person_id: Option<PersonId>) -> _ {
  comment_actions::table.on(
    comment_actions::comment_id
      .eq(comment::id)
      .and(comment_actions::person_id.nullable().eq(my_person_id)),
  )
}

#[diesel::dsl::auto_type]
pub fn my_person_actions_join(my_person_id: Option<PersonId>) -> _ {
  person_actions::table.on(
    person_actions::target_id
      .eq(person::id)
      .and(person_actions::person_id.nullable().eq(my_person_id)),
  )
}

#[diesel::dsl::auto_type]
pub fn my_local_user_admin_join(my_person_id: Option<PersonId>) -> _ {
  local_user::table.on(
    local_user::person_id
      .nullable()
      .eq(my_person_id)
      .and(local_user::admin.eq(true)),
  )
}

#[diesel::dsl::auto_type]
pub fn my_multi_community_follower_join(my_person_id: Option<PersonId>) -> _ {
  multi_community_follow::table.on(
    multi_community_follow::multi_community_id
      .eq(multi_community::id)
      .and(
        multi_community_follow::person_id
          .nullable()
          .eq(my_person_id),
      ),
  )
}

#[diesel::dsl::auto_type]
pub fn creator_community_actions_join() -> _ {
  creator_community_actions.on(
    creator_community_actions
      .field(community_actions::community_id)
      .eq(community::id)
      .and(
        creator_community_actions
          .field(community_actions::person_id)
          .eq(person::id),
      ),
  )
}
