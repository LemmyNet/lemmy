use crate::structs::PrivateMessageView;
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases,
  newtypes::PrivateMessageId,
  schema::{instance_actions, person, person_actions, private_message},
  utils::{actions, get_conn, DbPool},
};

impl PrivateMessageView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    private_message_id: PrivateMessageId,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    private_message::table
      .find(private_message_id)
      .inner_join(person::table.on(private_message::creator_id.eq(person::id)))
      .inner_join(
        aliases::person1.on(private_message::recipient_id.eq(aliases::person1.field(person::id))),
      )
      .left_join(actions(
        person_actions::table,
        Some(aliases::person1.field(person::id)),
        private_message::creator_id,
      ))
      .left_join(actions(
        instance_actions::table,
        Some(aliases::person1.field(person::id)),
        person::instance_id,
      ))
      .select((
        private_message::all_columns,
        person::all_columns,
        aliases::person1.fields(person::all_columns),
      ))
      .first(conn)
      .await
  }
}
