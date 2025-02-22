use crate::{
  newtypes::{PersonId, PrivateMessageId, PrivateMessageReportId},
  schema::private_message_report::dsl::{private_message_report, resolved, resolver_id, updated},
  source::private_message_report::{PrivateMessageReport, PrivateMessageReportForm},
  traits::Reportable,
  utils::{get_conn, DbPool},
};
use chrono::Utc;
use diesel::{
  dsl::{insert_into, update},
  result::Error,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_utils::error::{FederationError, LemmyResult};

impl Reportable for PrivateMessageReport {
  type Form = PrivateMessageReportForm;
  type IdType = PrivateMessageReportId;
  type ObjectIdType = PrivateMessageId;

  async fn report(
    pool: &mut DbPool<'_>,
    pm_report_form: &PrivateMessageReportForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(private_message_report)
      .values(pm_report_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn resolve(
    pool: &mut DbPool<'_>,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    update(private_message_report.find(report_id))
      .set((
        resolved.eq(true),
        resolver_id.eq(by_resolver_id),
        updated.eq(Utc::now()),
      ))
      .execute(conn)
      .await
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
  ) -> Result<usize, Error> {
    Err(Error::NotFound)
  }

  async fn unresolve(
    pool: &mut DbPool<'_>,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    update(private_message_report.find(report_id))
      .set((
        resolved.eq(false),
        resolver_id.eq(by_resolver_id),
        updated.eq(Utc::now()),
      ))
      .execute(conn)
      .await
  }
}
