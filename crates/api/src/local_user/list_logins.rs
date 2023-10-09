use actix_web::web::{Data, Json};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::login_token::LoginToken;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;

pub async fn list_logins(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<Vec<LoginToken>>, LemmyError> {
  let logins = LoginToken::list(&mut context.pool(), local_user_view.local_user.id).await?;

  Ok(Json(logins))
}
