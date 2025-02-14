use crate::{
  diesel::{ExpressionMethods, QueryDsl, SelectableHelper},
  newtypes::PaginationCursor,
  schema::person_saved_combined,
  source::combined::person_saved::PersonSavedCombined,
  traits::PageCursorReader,
  utils::{get_conn, DbPool},
};
use diesel_async::RunQueryDsl;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[async_trait]
impl PageCursorReader for PersonSavedCombined {
  async fn from_cursor(cursor: &PaginationCursor, pool: &mut DbPool<'_>) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    let (prefix, id) = cursor.prefix_and_id()?;

    let mut query = person_saved_combined::table
      .select(Self::as_select())
      .into_boxed();

    query = match prefix {
      'C' => query.filter(person_saved_combined::comment_id.eq(id)),
      'P' => query.filter(person_saved_combined::post_id.eq(id)),
      _ => return Err(LemmyErrorType::CouldntParsePaginationToken.into()),
    };
    let token = query.first(conn).await?;

    Ok(token)
  }
}
