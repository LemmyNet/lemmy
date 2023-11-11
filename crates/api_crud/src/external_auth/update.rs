use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  external_auth::{ExternalAuthResponse, EditExternalAuth},
  utils::is_admin,
};
use lemmy_db_schema::source::{
  external_auth::{ExternalAuth, ExternalAuthUpdateForm},
  local_site::LocalSite,
};
use lemmy_db_views::structs::{ExternalAuthView, LocalUserView};
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn update_external_auth(
  data: Json<EditExternalAuth>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<ExternalAuthResponse>, LemmyError> {
  let local_site = LocalSite::read(&mut context.pool()).await?;
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let clonedData = data.clone();
  let mut external_auth_form = ExternalAuthUpdateForm::builder()
    .local_site_id(local_site.id)
    .display_name(clonedData.display_name.into())
    .auth_type(data.auth_type.to_string())
    .auth_endpoint(clonedData.auth_endpoint.into())
    .token_endpoint(clonedData.token_endpoint.into())
    .user_endpoint(clonedData.user_endpoint.into())
    .id_attribute(data.id_attribute.to_string())
    .issuer(data.issuer.to_string())
    .client_id(data.client_id.to_string())
    .scopes(data.scopes.to_string());

  if data.client_secret != "" {
    external_auth_form = external_auth_form.client_secret(Some(data.client_secret.to_string()));
  } else {
    external_auth_form = external_auth_form.client_secret(None);
  }
  let external_auth = ExternalAuth::update(&mut context.pool(), data.id, &external_auth_form.build()).await?;
  let view = ExternalAuthView::get(&mut context.pool(), external_auth.id).await?;
  Ok(Json(ExternalAuthResponse { external_auth: view }))
}
