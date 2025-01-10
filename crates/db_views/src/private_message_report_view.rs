use crate::structs::PrivateMessageReportView;
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl};
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
    private_message_report::table
      .find(report_id)
      .inner_join(private_message::table)
      .inner_join(person::table.on(private_message::creator_id.eq(person::id)))
      .inner_join(
        aliases::person1
          .on(private_message_report::creator_id.eq(aliases::person1.field(person::id))),
      )
      .left_join(
        aliases::person2.on(
          private_message_report::resolver_id.eq(aliases::person2.field(person::id).nullable()),
        ),
      )
      .select((
        private_message_report::all_columns,
        private_message::all_columns,
        person::all_columns,
        aliases::person1.fields(person::all_columns),
        aliases::person2.fields(person::all_columns).nullable(),
      ))
      .first(conn)
      .await
  }
}
