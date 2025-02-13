use crate::{
  diesel::{ExpressionMethods, QueryDsl, SelectableHelper},
  newtypes::PaginationCursor,
  schema::search_combined,
  source::combined::search::SearchCombined,
  traits::PageCursorReader,
  utils::DbConn,
};
use diesel_async::RunQueryDsl;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[async_trait]
impl PageCursorReader for SearchCombined {
  async fn from_cursor(cursor: PaginationCursor, conn: &mut DbConn<'_>) -> LemmyResult<Self> {
    let (prefix, id) = cursor.prefix_and_id()?;

    let mut query = search_combined::table
      .select(Self::as_select())
      .into_boxed();

    query = match prefix {
      'P' => query.filter(search_combined::post_id.eq(id)),
      'C' => query.filter(search_combined::comment_id.eq(id)),
      'O' => query.filter(search_combined::community_id.eq(id)),
      'E' => query.filter(search_combined::person_id.eq(id)),
      _ => return Err(LemmyErrorType::CouldntParsePaginationToken.into()),
    };
    let token = query.first(conn).await?;

    Ok(token)
  }
}
