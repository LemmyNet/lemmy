use actix_web::web::{Data, Json};
use lemmy_api_common::{context::LemmyContext, person::BannedPersonsResponse, utils::is_admin};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::error::LemmyResult;

pub async fn list_banned_users(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<BannedPersonsResponse>> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let banned = PersonView::banned(&mut context.pool()).await?;

  Ok(Json(BannedPersonsResponse { banned }))
}
