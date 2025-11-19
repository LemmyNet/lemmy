use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use lemmy_db_schema_file::{
  InstanceId,
  PersonId,
  aliases,
  aliases::creator_community_actions,
  joins::{
    creator_community_instance_actions_join,
    creator_home_instance_actions_join,
    creator_local_instance_actions_join,
  },
  schema::{
    comment,
    comment_actions,
    comment_report,
    community,
    community_actions,
    community_report,
    local_user,
    person,
    person_actions,
    post,
    post_actions,
    post_report,
    private_message,
    private_message_report,
    report_combined,
  },
};

#[diesel::dsl::auto_type(no_type_alias)]
pub fn report_combined_joins(my_person_id: PersonId, local_instance_id: InstanceId) -> _ {
  // The item creator needs to be person::id, otherwise all the creator actions like
  // creator_banned will be wrong.
  let item_creator = person::id;
  let report_creator = aliases::person1.field(person::id);

  let resolver = aliases::person2.field(person::id).nullable();

  let comment_join = comment::table.on(comment_report::comment_id.eq(comment::id));
  let private_message_join =
    private_message::table.on(private_message_report::private_message_id.eq(private_message::id));

  let post_join = post::table.on(
    post_report::post_id
      .eq(post::id)
      .or(comment::post_id.eq(post::id)),
  );

  let community_actions_join = community_actions::table.on(
    community_actions::community_id
      .eq(community::id)
      .and(community_actions::person_id.eq(my_person_id)),
  );

  let report_creator_join = aliases::person1.on(
    post_report::creator_id
      .eq(report_creator)
      .or(comment_report::creator_id.eq(report_creator))
      .or(private_message_report::creator_id.eq(report_creator))
      .or(community_report::creator_id.eq(report_creator)),
  );

  let item_creator_join = person::table.on(
    post::creator_id
      .eq(item_creator)
      .or(comment::creator_id.eq(item_creator))
      .or(private_message::creator_id.eq(item_creator)),
  );

  let resolver_join = aliases::person2.on(
    private_message_report::resolver_id
      .eq(resolver)
      .or(post_report::resolver_id.eq(resolver))
      .or(comment_report::resolver_id.eq(resolver))
      .or(community_report::resolver_id.eq(resolver)),
  );

  let community_join = community::table.on(
    community_report::community_id
      .eq(community::id)
      .or(post::community_id.eq(community::id)),
  );

  let local_user_join = local_user::table.on(
    item_creator
      .eq(local_user::person_id)
      .and(local_user::admin.eq(true)),
  );

  let creator_community_actions_join = creator_community_actions.on(
    creator_community_actions
      .field(community_actions::community_id)
      .eq(post::community_id)
      .and(
        creator_community_actions
          .field(community_actions::person_id)
          .eq(item_creator),
      ),
  );
  let creator_local_instance_actions_join: creator_local_instance_actions_join =
    creator_local_instance_actions_join(local_instance_id);

  let post_actions_join = post_actions::table.on(
    post_actions::post_id
      .eq(post::id)
      .and(post_actions::person_id.eq(my_person_id)),
  );

  let person_actions_join = person_actions::table.on(
    person_actions::target_id
      .eq(item_creator)
      .and(person_actions::person_id.eq(my_person_id)),
  );

  let comment_actions_join = comment_actions::table.on(
    comment_actions::comment_id
      .eq(comment::id)
      .and(comment_actions::person_id.eq(my_person_id)),
  );

  report_combined::table
    .left_join(post_report::table)
    .left_join(comment_report::table)
    .left_join(private_message_report::table)
    .left_join(community_report::table)
    .inner_join(report_creator_join)
    .left_join(comment_join)
    .left_join(private_message_join)
    .left_join(post_join)
    .left_join(item_creator_join)
    .left_join(resolver_join)
    .left_join(community_join)
    .left_join(creator_community_actions_join)
    .left_join(creator_home_instance_actions_join())
    .left_join(creator_local_instance_actions_join)
    .left_join(creator_community_instance_actions_join())
    .left_join(local_user_join)
    .left_join(community_actions_join)
    .left_join(post_actions_join)
    .left_join(person_actions_join)
    .left_join(comment_actions_join)
}
