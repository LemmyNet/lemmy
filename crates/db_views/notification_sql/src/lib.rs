use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl};
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
    .inner_join(recipient_join)
    .inner_join(item_creator_join)
    .left_join(modlog::table)
    .left_join(comment::table)
    .left_join(post::table)
    .left_join(community::table)
    .left_join(instance::table)
    .left_join(image_details_join())
    .left_join(private_message::table)
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
}
