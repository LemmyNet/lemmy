use crate::structs::LocalImageView;
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::LocalUserId,
  schema::{local_image, local_user, person},
  utils::{get_conn, limit_and_offset, DbPool},
};

impl LocalImageView {
  async fn get_all_helper(
    pool: &mut DbPool<'_>,
    user_id: Option<LocalUserId>,
    page: Option<i64>,
    limit: Option<i64>,
    ignore_page_limits: bool,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut query = local_image::table
      .inner_join(local_user::table)
      .inner_join(person::table.on(local_user::person_id.eq(person::id)))
      .select((local_image::all_columns, person::all_columns))
      .order_by(local_image::published.desc())
      .into_boxed();

    if let Some(user_id) = user_id {
      query = query.filter(local_image::local_user_id.eq(user_id))
    }

    if !ignore_page_limits {
      let (limit, offset) = limit_and_offset(page, limit)?;
      query = query.limit(limit).offset(offset);
    }

    query.load::<LocalImageView>(conn).await
  }

  pub async fn get_all_paged_by_local_user_id(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    Self::get_all_helper(pool, Some(user_id), page, limit, false).await
  }

  pub async fn get_all_by_local_user_id(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
  ) -> Result<Vec<Self>, Error> {
    Self::get_all_helper(pool, Some(user_id), None, None, true).await
  }

  pub async fn get_all(
    pool: &mut DbPool<'_>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    Self::get_all_helper(pool, None, page, limit, false).await
  }
}
