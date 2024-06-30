use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{context::LemmyContext, oauth_provider::EditOAuthProvider, utils::is_admin};
use lemmy_db_schema::{
  source::oauth_provider::{OAuthProvider, OAuthProviderUpdateForm, UnsafeOAuthProvider},
  traits::Crud,
  utils::naive_now,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;
use url::Url;

#[tracing::instrument(skip(context))]
pub async fn update_oauth_provider(
  data: Json<EditOAuthProvider>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<OAuthProvider>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let cloned_data = data.clone();
  let oauth_provider_form = OAuthProviderUpdateForm::builder()
    .display_name(cloned_data.display_name)
    .authorization_endpoint(Url::parse(&cloned_data.authorization_endpoint)?.into())
    .token_endpoint(Url::parse(&cloned_data.token_endpoint)?.into())
    .userinfo_endpoint(Url::parse(&cloned_data.userinfo_endpoint)?.into())
    .id_claim(data.id_claim.to_string())
    .name_claim(data.name_claim.to_string())
    .client_secret(if !data.client_secret.is_empty() {
      Some(data.client_secret.to_string())
    } else {
      None
    })
    .scopes(data.scopes.to_string())
    .auto_verify_email(data.auto_verify_email)
    .auto_approve_application(data.auto_approve_application)
    .account_linking_enabled(data.account_linking_enabled)
    .enabled(data.enabled)
    .updated(naive_now());

  let update_result =
    UnsafeOAuthProvider::update(&mut context.pool(), data.id, &oauth_provider_form.build()).await?;
  let unsafe_oauth_provider =
    UnsafeOAuthProvider::get(&mut context.pool(), update_result.id).await?;
  Ok(Json(OAuthProvider::from_unsafe(&unsafe_oauth_provider)))
}
