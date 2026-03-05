use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  dsl::not,
};
use lemmy_db_schema_file::{
  InstanceId,
  PersonId,
  aliases,
  joins::{
    creator_community_actions_join,
    creator_home_instance_actions_join,
    creator_local_instance_actions_join,
    creator_local_user_admin_join,
    image_details_join,
    my_comment_actions_join,
    my_community_actions_join,
    my_instance_communities_actions_join,
    my_instance_persons_actions_join_1,
    my_local_user_admin_join,
    my_person_actions_join,
    my_post_actions_join,
  },
  schema::{comment, community, instance, modlog, notification, person, post, private_message},
};

#[diesel::dsl::auto_type(no_type_alias)]
pub fn notification_joins(person_id: PersonId, instance_id: InstanceId) -> _ {
  let item_creator_join = person::table.on(notification::creator_id.eq(person::id));

  // No need to join on `modlog::target_person_id` as it is identical to
  // `notification::recipient_id`.
  let recipient_person = aliases::person1.field(person::id);
  let recipient_join = aliases::person1.on(notification::recipient_id.eq(recipient_person));

  let comment_join = comment::table.on(
    notification::comment_id
      .eq(comment::id.nullable())
      // Filter out the deleted / removed
      .and(not(comment::deleted))
      .and(not(comment::removed))
      .or(modlog::target_comment_id.eq(comment::id.nullable())),
  );

  let post_join = post::table.on(
    notification::post_id
      .eq(post::id.nullable())
      .or(comment::post_id.eq(post::id))
      // Filter out the deleted / removed
      .and(not(post::deleted))
      .and(not(post::removed))
      .or(modlog::target_post_id.eq(post::id.nullable())),
  );

  let community_join = community::table.on(
    post::community_id
      .eq(community::id)
      .or(modlog::target_community_id.eq(community::id.nullable())),
  );

  // This could be a simple join, but you need to check for deleted here
  let private_message_join = private_message::table.on(
    notification::private_message_id
      .eq(private_message::id.nullable())
      .and(not(private_message::deleted))
      .and(not(private_message::removed)),
  );

  let instance_join = instance::table.on(modlog::target_instance_id.eq(instance::id.nullable()));

  let my_community_actions_join: my_community_actions_join =
    my_community_actions_join(Some(person_id));
  let my_post_actions_join: my_post_actions_join = my_post_actions_join(Some(person_id));
  let my_comment_actions_join: my_comment_actions_join = my_comment_actions_join(Some(person_id));
  let my_instance_communities_actions_join: my_instance_communities_actions_join =
    my_instance_communities_actions_join(Some(person_id));
  let my_instance_persons_actions_join_1: my_instance_persons_actions_join_1 =
    my_instance_persons_actions_join_1(Some(person_id));
  let my_person_actions_join: my_person_actions_join = my_person_actions_join(Some(person_id));
  let creator_local_instance_actions_join: creator_local_instance_actions_join =
    creator_local_instance_actions_join(instance_id);
  let my_local_user_admin_join: my_local_user_admin_join =
    my_local_user_admin_join(Some(person_id));

  // Note: avoid adding any more joins here as it will significantly slow down compilation.
  notification::table
    .left_join(modlog::table)
    .left_join(private_message_join)
    .left_join(comment_join)
    .left_join(post_join)
    .left_join(community_join)
    .inner_join(item_creator_join)
    .inner_join(recipient_join)
    .left_join(image_details_join())
    .left_join(creator_community_actions_join())
    .left_join(creator_local_user_admin_join())
    .left_join(creator_home_instance_actions_join())
    .left_join(creator_local_instance_actions_join)
    .left_join(my_local_user_admin_join)
    .left_join(my_community_actions_join)
    .left_join(my_instance_communities_actions_join)
    .left_join(my_instance_persons_actions_join_1)
    .left_join(my_post_actions_join)
    .left_join(my_person_actions_join)
    .left_join(my_comment_actions_join)
    .left_join(instance_join)
}
