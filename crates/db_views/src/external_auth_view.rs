use crate::structs::ExternalAuthView;
use diesel::{result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{ExternalAuthId, LocalSiteId},
  schema::{external_auth},
  source::external_auth::ExternalAuth,
  utils::{get_conn, DbPool},
};

impl ExternalAuthView {
  pub async fn get(pool: &mut DbPool<'_>, external_auth_id: ExternalAuthId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let external_auths = external_auth::table
      .find(external_auth_id)
      .select(external_auth::all_columns)
      .load::<ExternalAuth>(conn)
      .await?;
    if let Some(external_auth) = ExternalAuthView::from_tuple_to_vec(external_auths)
      .into_iter()
      .next()
    {
      Ok(external_auth)
    } else {
      Err(diesel::result::Error::NotFound)
    }
  }

  // client_secret is in its own function because it should never be sent to any frontends,
  // and will only be needed when performing an oauth request by the server
  pub async fn get_client_secret(pool: &mut DbPool<'_>, external_auth_id: ExternalAuthId) -> Result<String, Error> {
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

  pub async fn get_all(
    pool: &mut DbPool<'_>,
    for_local_site_id: LocalSiteId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let external_auths = external_auth::table
      .filter(external_auth::local_site_id.eq(for_local_site_id))
      .order(external_auth::id)
      .select(external_auth::all_columns)
      .load::<ExternalAuth>(conn)
      .await?;

    Ok(ExternalAuthView::from_tuple_to_vec(external_auths))
  }

  fn from_tuple_to_vec(items: Vec<ExternalAuth>) -> Vec<Self> {
    let mut result = Vec::new();
    for item in &items {
      result.push(ExternalAuthView {
        // Can't just clone entire object because client_secret must be stripped
        external_auth: ExternalAuth {
          id: item.id.clone(),
          local_site_id: item.local_site_id.clone(),
          display_name: item.display_name.clone(),
          auth_type: item.auth_type.clone(),
          auth_endpoint: item.auth_endpoint.clone(),
          token_endpoint: item.token_endpoint.clone(),
          user_endpoint: item.user_endpoint.clone(),
          id_attribute: item.id_attribute.clone(),
          issuer: item.issuer.clone(),
          client_id: item.client_id.clone(),
          client_secret: String::new(),
          scopes: item.scopes.clone(),
          published: item.published.clone(),
          updated: item.updated.clone(),
        },
      });
    }
    result
  }
}
