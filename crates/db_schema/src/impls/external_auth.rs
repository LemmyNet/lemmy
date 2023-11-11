use crate::{
  newtypes::ExternalAuthId,
  schema::{
    external_auth::dsl::external_auth,
  },
  source::{
    external_auth::{ExternalAuth, ExternalAuthInsertForm, ExternalAuthUpdateForm},
  },
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

impl ExternalAuth {
  pub async fn create(pool: &mut DbPool<'_>, form: &ExternalAuthInsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(external_auth)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn update(
    pool: &mut DbPool<'_>,
    external_auth_id: ExternalAuthId,
    form: &ExternalAuthUpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(external_auth.find(external_auth_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn delete(pool: &mut DbPool<'_>, external_auth_id: ExternalAuthId) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(external_auth.find(external_auth_id))
      .execute(conn)
      .await
  }
}
  