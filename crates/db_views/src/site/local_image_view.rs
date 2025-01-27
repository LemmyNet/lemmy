use crate::structs::LocalImageView;
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::LocalUserId,
  schema::{local_image, local_user, person},
  utils::{get_conn, limit_and_offset, DbPool},
};

#[diesel::dsl::auto_type]
fn joins() -> _ {
  local_image::table
    .inner_join(local_user::table)
    .inner_join(person::table.on(local_user::person_id.eq(person::id)))
}

type SelectionType = (
  <local_image::table as diesel::Table>::AllColumns,
  <person::table as diesel::Table>::AllColumns,
);

const SELECTION: SelectionType = (local_image::all_columns, person::all_columns);

impl LocalImageView {
  pub async fn get_all_paged_by_local_user_id(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let (limit, offset) = limit_and_offset(page, limit)?;

    joins()
      .filter(local_image::local_user_id.eq(user_id))
      .select(SELECTION)
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
    joins()
      .filter(local_image::local_user_id.eq(user_id))
      .select(SELECTION)
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
    joins()
      .select(SELECTION)
      .limit(limit)
      .offset(offset)
      .load::<Self>(conn)
      .await
  }
}
