use activitypub_federation::config::Data;
use actix_web::web::{Json, Path};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::{newtypes::TaglineId, source::tagline::Tagline, traits::Crud};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::LemmyError;

pub async fn delete_tagline(
  id: Path<TaglineId>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<SuccessResponse>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  Tagline::delete(&mut context.pool(), id.into_inner()).await?;

  Ok(Json(SuccessResponse::default()))
}
