use crate::{
  diesel::{ExpressionMethods, QueryDsl, SelectableHelper},
  newtypes::PaginationCursor,
  schema::report_combined,
  source::combined::report::ReportCombined,
  traits::PaginationCursorReader,
  utils::{get_conn, DbPool},
};
use diesel_async::RunQueryDsl;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[async_trait]
impl PaginationCursorReader for ReportCombined {
  async fn from_cursor(cursor: &PaginationCursor, pool: &mut DbPool<'_>) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    let (prefix, id) = cursor.prefix_and_id()?;

    let mut query = report_combined::table
      .select(Self::as_select())
      .into_boxed();

    query = match prefix {
      'C' => query.filter(report_combined::comment_report_id.eq(id)),
      'P' => query.filter(report_combined::post_report_id.eq(id)),
      'M' => query.filter(report_combined::private_message_report_id.eq(id)),
      'Y' => query.filter(report_combined::community_report_id.eq(id)),
      _ => return Err(LemmyErrorType::CouldntParsePaginationToken.into()),
    };
    let token = query.first(conn).await?;

    Ok(token)
  }
}
