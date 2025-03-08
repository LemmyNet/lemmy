use crate::structs::PrivateMessageReportView;
use diesel::{
  result::Error,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases,
  newtypes::PrivateMessageReportId,
  schema::{person, private_message, private_message_report},
  utils::{get_conn, DbPool},
};

impl PrivateMessageReportView {
  /// returns the PrivateMessageReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub async fn read(
    pool: &mut DbPool<'_>,
    report_id: PrivateMessageReportId,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    let recipient_id = aliases::person1.field(person::id);
    let resolver_id = aliases::person2.field(person::id);

    let report_creator_join = person::table.on(private_message_report::creator_id.eq(person::id));
    let private_message_creator_join =
      aliases::person1.on(private_message::creator_id.eq(recipient_id));
    let resolver_join =
      aliases::person2.on(private_message_report::resolver_id.eq(resolver_id.nullable()));

    private_message_report::table
      .find(report_id)
      .inner_join(private_message::table)
      .inner_join(report_creator_join)
      .inner_join(private_message_creator_join)
      .left_join(resolver_join)
      .select(Self::as_select())
      .first(conn)
      .await
  }
}
