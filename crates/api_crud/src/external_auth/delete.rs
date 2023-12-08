use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  external_auth::DeleteExternalAuth,
  utils::is_admin,
  SuccessResponse,
};
use lemmy_db_schema::source::external_auth::ExternalAuth;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn delete_external_auth(
  data: Json<DeleteExternalAuth>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<SuccessResponse>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;
  ExternalAuth::delete(&mut context.pool(), data.id).await?;
  Ok(Json(SuccessResponse::default()))
}
