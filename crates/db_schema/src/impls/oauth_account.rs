use crate::{
  newtypes::OAuthAccountId,
  schema::oauth_account,
  source::oauth_account::{OAuthAccount, OAuthAccountInsertForm, OAuthAccountUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Crud for OAuthAccount {
  type InsertForm = OAuthAccountInsertForm;
  type UpdateForm = OAuthAccountUpdateForm;
  type IdType = OAuthAccountId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(oauth_account::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  async fn update(
    pool: &mut DbPool<'_>,
    oauth_account_id: OAuthAccountId,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(oauth_account::table.find(oauth_account_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl OAuthAccount {
  pub async fn get(pool: &mut DbPool<'_>, oauth_account_id: OAuthAccountId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let oauth_accounts = oauth_account::table
      .find(oauth_account_id)
      .select(oauth_account::all_columns)
      .limit(1)
      .load::<OAuthAccount>(conn)
      .await?;
    if let Some(oauth_account) = oauth_accounts.into_iter().next() {
      Ok(oauth_account)
    } else {
      Err(diesel::result::Error::NotFound)
    }
  }
}
