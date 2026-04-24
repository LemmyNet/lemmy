use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::{
  newtypes::LocalUserId,
  source::local_user_invite::{LocalUserInvite, invitation_keys as key},
  utils::limit_fetch,
};
use lemmy_db_schema_file::schema::local_user_invite;
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  pagination::{PagedResponse, PaginationCursor, PaginationCursorConversion, paginate_response},
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[derive(Default)]
pub struct LocalUserInviteQuery {
  pub local_user_id: LocalUserId,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
}

impl LocalUserInviteQuery {
  pub async fn list(self, pool: &mut DbPool<'_>) -> LemmyResult<PagedResponse<LocalUserInvite>> {
    let limit = limit_fetch(self.limit, None)?;

    let mut query = local_user_invite::table
      .select(LocalUserInvite::as_select())
      .limit(limit)
      .into_boxed();

    query = query.filter(local_user_invite::local_user_id.eq(self.local_user_id));

    let paginated_query =
      LocalUserInvite::paginate(query, &self.page_cursor, SortDirection::Asc, pool)
        .await?
        .then_order_by(key::published_at)
        .then_order_by(key::id);

    let conn = &mut get_conn(pool).await?;
    let res = paginated_query
      .load::<LocalUserInvite>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)?;
    paginate_response(res, limit, self.page_cursor)
  }

  pub async fn count(self, pool: &mut DbPool<'_>) -> LemmyResult<i64> {
    use diesel::dsl::count_star;

    let conn = &mut get_conn(pool).await?;
    let mut query = local_user_invite::table.select(count_star()).into_boxed();

    query = query.filter(local_user_invite::local_user_id.eq(self.local_user_id));

    query
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}
