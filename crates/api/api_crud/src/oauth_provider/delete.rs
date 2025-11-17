use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::source::oauth_provider::OAuthProvider;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::{DeleteOAuthProvider, SuccessResponse};
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::LemmyError;

pub async fn delete_oauth_provider(
  data: Json<DeleteOAuthProvider>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<SuccessResponse>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  OAuthProvider::delete(&mut context.pool(), data.id).await?;

  Ok(Json(SuccessResponse::default()))
}
