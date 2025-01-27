use crate::structs::LocalImageView;
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::LocalUserId,
  schema::{local_image, local_user, person},
  utils::{get_conn, limit_and_offset, DbPool},
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
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let (limit, offset) = limit_and_offset(page, limit)?;

    Self::joins()
      .filter(local_image::local_user_id.eq(user_id))
      .select(Self::as_select())
      .limit(limit)
      .offset(offset)
      .load::<Self>(conn)
      .await
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
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let (limit, offset) = limit_and_offset(page, limit)?;
    Self::joins()
      .select(Self::as_select())
      .limit(limit)
      .offset(offset)
      .load::<Self>(conn)
      .await
  }
}
