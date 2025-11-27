use crate::LocalImageView;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::{
  source::images::{LocalImage, local_image_keys as key},
  utils::limit_fetch,
};
use lemmy_db_schema_file::{
  PersonId,
  schema::{local_image, person, post},
};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  pagination::{
    CursorData,
    PagedResponse,
    PaginationCursor,
    PaginationCursorConversion,
    paginate_response,
  },
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl LocalImageView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    local_image::table
      .inner_join(person::table)
      .left_join(post::table)
  }

  pub async fn get_all_paged_by_person_id(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    cursor_data: Option<PaginationCursor>,
    limit: Option<i64>,
  ) -> LemmyResult<PagedResponse<Self>> {
    let limit = limit_fetch(limit, None)?;

    let query = Self::joins()
      .filter(local_image::person_id.eq(person_id))
      .select(Self::as_select())
      .limit(limit)
      .into_boxed();

    let paginated_query = Self::paginate(query, &cursor_data, SortDirection::Asc, pool, None)
      .await?
      .then_order_by(key::pictrs_alias);

    let conn = &mut get_conn(pool).await?;
    let res = paginated_query
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)?;

    paginate_response(res, limit, cursor_data)
  }

  pub async fn get_all_by_person_id(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(local_image::person_id.eq(person_id))
      .select(Self::as_select())
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn get_all_paged(
    pool: &mut DbPool<'_>,
    cursor_data: Option<PaginationCursor>,
    limit: Option<i64>,
  ) -> LemmyResult<PagedResponse<Self>> {
    let limit = limit_fetch(limit, None)?;

    let query = Self::joins()
      .select(Self::as_select())
      .limit(limit)
      .into_boxed();

    let paginated_query =
      Self::paginate(query, &cursor_data, SortDirection::Asc, pool, None).await?;
    let conn = &mut get_conn(pool).await?;
    let res = paginated_query
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)?;
    paginate_response(res, limit, cursor_data)
  }
}

impl PaginationCursorConversion for LocalImageView {
  type PaginatedType = LocalImage;
  fn to_cursor(&self) -> CursorData {
    // Use pictrs alias
    CursorData::new_plain(self.local_image.pictrs_alias.clone())
  }

  async fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    let conn = &mut get_conn(pool).await?;

    // This isn't an id, but a string
    let alias = cursor.plain();

    let token = local_image::table
      .select(Self::PaginatedType::as_select())
      .filter(local_image::pictrs_alias.eq(alias))
      .first(conn)
      .await?;

    Ok(token)
  }
}
