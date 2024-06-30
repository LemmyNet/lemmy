use crate::{
  newtypes::OAuthProviderId,
  schema::oauth_provider,
  source::oauth_provider::{
    OAuthProvider,
    OAuthProviderInsertForm,
    OAuthProviderUpdateForm,
    UnsafeOAuthProvider,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Crud for UnsafeOAuthProvider {
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

impl UnsafeOAuthProvider {
  pub async fn get(
    pool: &mut DbPool<'_>,
    oauth_provider_id: OAuthProviderId,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let oauth_providers = oauth_provider::table
      .find(oauth_provider_id)
      .select(oauth_provider::all_columns)
      .limit(1)
      .load::<UnsafeOAuthProvider>(conn)
      .await?;
    if let Some(oauth_provider) = oauth_providers.into_iter().next() {
      Ok(oauth_provider)
    } else {
      Err(diesel::result::Error::NotFound)
    }
  }

  pub async fn get_all(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let oauth_providers = oauth_provider::table
      .order(oauth_provider::id)
      .select(oauth_provider::all_columns)
      .load::<UnsafeOAuthProvider>(conn)
      .await?;

    Ok(oauth_providers)
  }
}

impl OAuthProvider {
  pub async fn get_all(pool: &mut DbPool<'_>) -> Result<Vec<Option<Self>>, Error> {
    let oauth_providers = UnsafeOAuthProvider::get_all(pool).await?;
    let mut result = Vec::<Option<OAuthProvider>>::new();

    for oauth_provider in &oauth_providers {
      result.push(Some(Self::from_unsafe(oauth_provider)));
    }

    Ok(result)
  }

  pub fn from_unsafe(unsafe_oauth_provider: &UnsafeOAuthProvider) -> Self {
    OAuthProvider {
      id: unsafe_oauth_provider.id,
      display_name: unsafe_oauth_provider.display_name.clone(),
      issuer: Some(unsafe_oauth_provider.issuer.clone()),
      authorization_endpoint: unsafe_oauth_provider.authorization_endpoint.clone(),
      token_endpoint: Some(unsafe_oauth_provider.token_endpoint.clone()),
      userinfo_endpoint: Some(unsafe_oauth_provider.userinfo_endpoint.clone()),
      id_claim: Some(unsafe_oauth_provider.id_claim.clone()),
      name_claim: Some(unsafe_oauth_provider.name_claim.clone()),
      client_id: unsafe_oauth_provider.client_id.clone(),
      scopes: unsafe_oauth_provider.scopes.clone(),
      auto_verify_email: Some(unsafe_oauth_provider.auto_verify_email),
      auto_approve_application: Some(unsafe_oauth_provider.auto_approve_application),
      account_linking_enabled: Some(unsafe_oauth_provider.account_linking_enabled),
      enabled: Some(unsafe_oauth_provider.enabled),
      published: Some(unsafe_oauth_provider.published),
      updated: unsafe_oauth_provider.updated,
    }
  }
}
