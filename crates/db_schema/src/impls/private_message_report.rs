use crate::{
  newtypes::{PersonId, PrivateMessageId, PrivateMessageReportId},
  source::private_message_report::{PrivateMessageReport, PrivateMessageReportForm},
  traits::Reportable,
  utils::{get_conn, DbPool},
};
use chrono::Utc;
use diesel::{
  dsl::{insert_into, update},
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::private_message_report;
use lemmy_utils::error::{FederationError, LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Reportable for PrivateMessageReport {
  type Form = PrivateMessageReportForm;
  type IdType = PrivateMessageReportId;
  type ObjectIdType = PrivateMessageId;

  async fn report(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(private_message_report::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreateReport)
  }

  async fn resolve(
    pool: &mut DbPool<'_>,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(private_message_report::table.find(report_id))
      .set((
        private_message_report::resolved.eq(true),
        private_message_report::resolver_id.eq(by_resolver_id),
        private_message_report::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntResolveReport)
  }
  async fn resolve_apub(
    _pool: &mut DbPool<'_>,
    _object_id: Self::ObjectIdType,
    _report_creator_id: PersonId,
    _resolver_id: PersonId,
  ) -> LemmyResult<usize> {
    Err(FederationError::Unreachable.into())
  }

  // TODO: this is unused because private message doesn't have remove handler
  async fn resolve_all_for_object(
    _pool: &mut DbPool<'_>,
    _pm_id_: PrivateMessageId,
    _by_resolver_id: PersonId,
  ) -> LemmyResult<usize> {
    Err(LemmyErrorType::NotFound.into())
  }

  async fn unresolve(
    pool: &mut DbPool<'_>,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(private_message_report::table.find(report_id))
      .set((
        private_message_report::resolved.eq(false),
        private_message_report::resolver_id.eq(by_resolver_id),
        private_message_report::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntResolveReport)
  }
}
