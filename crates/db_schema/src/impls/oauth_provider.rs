use crate::{
  newtypes::OAuthProviderId,
  source::oauth_provider::{
    OAuthProvider,
    OAuthProviderInsertForm,
    OAuthProviderUpdateForm,
    PublicOAuthProvider,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::oauth_provider;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Crud for OAuthProvider {
  type InsertForm = OAuthProviderInsertForm;
  type UpdateForm = OAuthProviderUpdateForm;
  type IdType = OAuthProviderId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(oauth_provider::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreateOauthProvider)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    oauth_provider_id: OAuthProviderId,
    form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(oauth_provider::table.find(oauth_provider_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdateOauthProvider)
  }
}

impl OAuthProvider {
  pub async fn get_all(pool: &mut DbPool<'_>) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    oauth_provider::table
      .order(oauth_provider::id)
      .select(oauth_provider::all_columns)
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
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

  pub async fn get_all_public(pool: &mut DbPool<'_>) -> LemmyResult<Vec<PublicOAuthProvider>> {
    OAuthProvider::get_all(pool)
      .await
      .map(Self::convert_providers_to_public)
  }
}
