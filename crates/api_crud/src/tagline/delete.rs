use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  tagline::DeleteTagline,
  utils::is_admin,
  SuccessResponse,
};
use lemmy_db_schema::{source::tagline::Tagline, traits::Crud};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn delete_tagline(
  data: Json<DeleteTagline>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<SuccessResponse>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  Tagline::delete(&mut context.pool(), data.id).await?;

  Ok(Json(SuccessResponse::default()))
}
