use crate::{
  newtypes::OAuthProviderId,
  schema::oauth_provider,
  source::oauth_provider::{
    OAuthProvider,
    OAuthProviderInsertForm,
    OAuthProviderUpdateForm,
    PublicOAuthProvider,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Crud for OAuthProvider {
  type InsertForm = OAuthProviderInsertForm;
  type UpdateForm = OAuthProviderUpdateForm;
  type IdType = OAuthProviderId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(oauth_provider::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    oauth_provider_id: OAuthProviderId,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(oauth_provider::table.find(oauth_provider_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl OAuthProvider {
  pub async fn get_all(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let oauth_providers = oauth_provider::table
      .order(oauth_provider::id)
      .select(oauth_provider::all_columns)
      .load::<OAuthProvider>(conn)
      .await?;

    Ok(oauth_providers)
  }

  pub fn convert_providers_to_public(
    oauth_providers: Vec<OAuthProvider>,
  ) -> Vec<PublicOAuthProvider> {
    oauth_providers
      .into_iter()
      .filter(|x| x.enabled)
      .map(PublicOAuthProvider)
      .collect()
  }

  pub async fn get_all_public(pool: &mut DbPool<'_>) -> Result<Vec<PublicOAuthProvider>, Error> {
    let oauth_providers = OAuthProvider::get_all(pool).await?;
    Ok(Self::convert_providers_to_public(oauth_providers))
  }
}
