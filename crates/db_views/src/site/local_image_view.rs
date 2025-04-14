use crate::structs::LocalImageView;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::{
  newtypes::{LocalUserId, PaginationCursor},
  source::images::{local_image_keys as key, LocalImage},
  traits::PaginationCursorBuilder,
  utils::{get_conn, limit_fetch, paginate, DbPool},
};
use lemmy_db_schema_file::schema::{local_image, local_user, person};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl LocalImageView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    local_image::table
      .inner_join(local_user::table)
      .inner_join(person::table.on(local_user::person_id.eq(person::id)))
  }

  pub async fn get_all_paged_by_local_user_id(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    cursor_data: Option<LocalImage>,
    page_back: Option<bool>,
    limit: Option<i64>,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(limit)?;

    let query = Self::joins()
      .filter(local_image::local_user_id.eq(user_id))
      .select(Self::as_select())
      .limit(limit)
      .into_boxed();

    let paginated_query = paginate(query, SortDirection::Asc, cursor_data, None, page_back)
      .then_order_by(key::pictrs_alias);

    paginated_query
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn get_all_by_local_user_id(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(local_image::local_user_id.eq(user_id))
      .select(Self::as_select())
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn get_all_paged(
    pool: &mut DbPool<'_>,
    cursor_data: Option<LocalImage>,
    page_back: Option<bool>,
    limit: Option<i64>,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(limit)?;

    let query = Self::joins()
      .select(Self::as_select())
      .limit(limit)
      .into_boxed();

    let paginated_query = paginate(query, SortDirection::Asc, cursor_data, None, page_back);
    paginated_query
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl PaginationCursorBuilder for LocalImageView {
  type CursorData = LocalImage;
  fn to_cursor(&self) -> PaginationCursor {
    // Use pictrs alias
    PaginationCursor(format!("A{}", self.local_image.pictrs_alias))
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::CursorData> {
    let conn = &mut get_conn(pool).await?;

    // This isn't an id, but a string
    let alias = cursor
      .0
      .split_at_checked(1)
      .ok_or(LemmyErrorType::CouldntParsePaginationToken)?
      .1;

    let token = local_image::table
      .select(Self::CursorData::as_select())
      .filter(local_image::pictrs_alias.eq(alias))
      .first(conn)
      .await?;

    Ok(token)
  }
}
