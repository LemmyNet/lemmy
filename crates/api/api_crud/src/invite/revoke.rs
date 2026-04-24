use actix_web::web::{Data, Json};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::local_user_invite::LocalUserInvite;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_local_user_invite::api::RevokeInvitation;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::LemmyResult;

pub async fn revoke_invitation(
  Json(data): Json<RevokeInvitation>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let pool = &mut context.pool();

  let local_user_id = local_user_view.local_user.id;

  LocalUserInvite::delete_by_token_and_user(pool, &local_user_id, &data.token).await?;

  Ok(Json(SuccessResponse::default()))
}
