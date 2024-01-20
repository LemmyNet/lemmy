use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  external_auth::CreateExternalAuth,
  utils::is_admin,
};
use lemmy_db_schema::{
  source::external_auth::{ExternalAuth, ExternalAuthInsertForm},
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn create_external_auth(
  data: Json<CreateExternalAuth>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<ExternalAuth>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let cloned_data = data.clone();
  let external_auth_form = ExternalAuthInsertForm::builder()
    .display_name(cloned_data.display_name.into())
    .auth_type(data.auth_type.to_string())
    .auth_endpoint(cloned_data.auth_endpoint.into())
    .token_endpoint(cloned_data.token_endpoint.into())
    .user_endpoint(cloned_data.user_endpoint.into())
    .id_attribute(cloned_data.id_attribute.into())
    .issuer(data.issuer.to_string())
    .client_id(data.client_id.to_string())
    .client_secret(data.client_secret.to_string())
    .scopes(data.scopes.to_string())
    .build();
  let external_auth = ExternalAuth::create(&mut context.pool(), &external_auth_form).await?;
  Ok(Json(external_auth))
}
