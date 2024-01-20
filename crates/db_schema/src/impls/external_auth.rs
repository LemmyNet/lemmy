use crate::{
  newtypes::ExternalAuthId,
  schema::external_auth::dsl::external_auth,
  source::external_auth::{ExternalAuth, ExternalAuthInsertForm, ExternalAuthUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{
  associations::HasTable,
  dsl::insert_into,
  result::Error,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Crud for ExternalAuth {
  type InsertForm = ExternalAuthInsertForm;
  type UpdateForm = ExternalAuthUpdateForm;
  type IdType = ExternalAuthId;

  async fn create(pool: &mut DbPool<'_>, form: &ExternalAuthInsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(external_auth)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  async fn update(
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
  async fn delete(pool: &mut DbPool<'_>, external_auth_id: ExternalAuthId) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(external_auth.find(external_auth_id))
      .execute(conn)
      .await
  }
}

impl ExternalAuth {
  pub async fn get(pool: &mut DbPool<'_>, external_auth_id: ExternalAuthId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let external_auths = external_auth::table
      .find(external_auth_id)
      .select(external_auth::all_columns)
      .load::<ExternalAuth>(conn)
      .await?;
    if let Some(external_auth) = external_auths.into_iter().next() {
      Ok(external_auth)
    } else {
      Err(diesel::result::Error::NotFound)
    }
  }

  // client_secret is in its own function because it should never be sent to any frontends,
  // and will only be needed when performing an oauth request by the server
  pub async fn get_client_secret(
    pool: &mut DbPool<'_>,
    external_auth_id: ExternalAuthId,
  ) -> Result<String, Error> {
    let conn = &mut get_conn(pool).await?;
    let external_auths = external_auth::table
      .find(external_auth_id)
      .select(external_auth::client_secret)
      .load::<String>(conn)
      .await?;
    if let Some(external_auth) = external_auths.into_iter().next() {
      Ok(external_auth)
    } else {
      Err(diesel::result::Error::NotFound)
    }
  }

  pub async fn get_all(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let external_auths = external_auth::table
      .order(external_auth::id)
      .select(external_auth::all_columns)
      .load::<ExternalAuth>(conn)
      .await?;

    Ok(external_auths)
  }
}
