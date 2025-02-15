use crate::structs::LocalImageView;
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use i_love_jesus::PaginatedQueryBuilder;
use lemmy_db_schema::{
  newtypes::{LocalUserId, PaginationCursor},
  schema::{local_image, local_user, person},
  source::images::LocalImage,
  traits::PageCursorBuilder,
  utils::{get_conn, DbPool},
};

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
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;

    let query = Self::joins()
      .filter(local_image::local_user_id.eq(user_id))
      .select(Self::as_select())
      .into_boxed();

    let mut query = PaginatedQueryBuilder::new(query);

    if page_back.unwrap_or_default() {
      query = query.before(cursor_data).limit_and_offset_from_end();
    } else {
      query = query.after(cursor_data);
    }

    query.load::<Self>(conn).await
  }

  pub async fn get_all_by_local_user_id(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(local_image::local_user_id.eq(user_id))
      .select(Self::as_select())
      .load::<Self>(conn)
      .await
  }

  pub async fn get_all(
    pool: &mut DbPool<'_>,
    cursor_data: Option<LocalImage>,
    page_back: Option<bool>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let query = Self::joins().select(Self::as_select()).into_boxed();

    let mut query = PaginatedQueryBuilder::new(query);

    if page_back.unwrap_or_default() {
      query = query.before(cursor_data).limit_and_offset_from_end();
    } else {
      query = query.after(cursor_data);
    }

    query.load::<Self>(conn).await
  }
}

impl PageCursorBuilder for LocalImageView {
  fn cursor(&self) -> PaginationCursor {
    // Use pictrs alias
    PaginationCursor(format!("A{}", self.local_image.pictrs_alias))
  }
}
