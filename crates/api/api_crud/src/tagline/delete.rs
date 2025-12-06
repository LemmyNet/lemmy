use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::source::tagline::Tagline;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::{DeleteTagline, SuccessResponse};
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::LemmyError;

pub async fn delete_tagline(
  Json(data): Json<DeleteTagline>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<SuccessResponse>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  Tagline::delete(&mut context.pool(), data.id).await?;

  Ok(Json(SuccessResponse::default()))
}
