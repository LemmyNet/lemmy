use crate::structs::PrivateMessageView;
use diesel::{
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases,
  newtypes::PrivateMessageId,
  schema::{instance_actions, person, person_actions, private_message},
  utils::{get_conn, DbPool},
};

impl PrivateMessageView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    let recipient_id = aliases::person1.field(person::id);

    let creator_join = person::table.on(private_message::creator_id.eq(person::id));
    let recipient_join = aliases::person1.on(private_message::recipient_id.eq(recipient_id));

    let person_actions_join = person_actions::table.on(
      person_actions::target_id
        .eq(private_message::creator_id)
        .and(person_actions::person_id.eq(recipient_id)),
    );

    let instance_actions_join = instance_actions::table.on(
      instance_actions::instance_id
        .eq(person::instance_id)
        .and(instance_actions::person_id.eq(recipient_id)),
    );

    private_message::table
      .inner_join(creator_join)
      .inner_join(recipient_join)
      .left_join(person_actions_join)
      .left_join(instance_actions_join)
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    private_message_id: PrivateMessageId,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(private_message::id.eq(private_message_id))
      .select(Self::as_select())
      .first(conn)
      .await
  }
}
