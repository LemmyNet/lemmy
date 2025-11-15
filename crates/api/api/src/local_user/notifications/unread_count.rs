use actix_web::web::{Data, Json};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_notification::{NotificationView, api::GetUnreadCountResponse};
use lemmy_utils::error::LemmyResult;

pub async fn unread_count(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetUnreadCountResponse>> {
  let show_bot_accounts = local_user_view.local_user.show_bot_accounts;
  let count = NotificationView::get_unread_count(
    &mut context.pool(),
    &local_user_view.person,
    show_bot_accounts,
  )
  .await?;

  Ok(Json(GetUnreadCountResponse { count }))
}
