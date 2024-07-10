use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  oauth_provider::DeleteOAuthProvider,
  utils::is_admin,
  SuccessResponse,
};
use lemmy_db_schema::{source::oauth_provider::UnsafeOAuthProvider, traits::Crud};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyError, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn delete_oauth_provider(
  data: Json<DeleteOAuthProvider>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<SuccessResponse>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;
  UnsafeOAuthProvider::delete(&mut context.pool(), data.id)
    .await
    .ok()
    .ok_or(LemmyErrorType::CouldntDeleteOauthProvider)?;
  Ok(Json(SuccessResponse::default()))
}
