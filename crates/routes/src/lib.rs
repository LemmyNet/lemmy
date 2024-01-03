use lemmy_api_common::{claims::Claims, context::LemmyContext, utils::check_user_valid};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt2, LemmyErrorType};

pub mod feeds;
pub mod images;
pub mod nodeinfo;
pub mod webfinger;

#[tracing::instrument(skip_all)]
async fn local_user_view_from_jwt(
  jwt: &str,
  context: &LemmyContext,
) -> Result<LocalUserView, LemmyError> {
  let local_user_id = Claims::validate(jwt, context)
    .await
    .with_lemmy_type(LemmyErrorType::NotLoggedIn)?;
  let local_user_view = LocalUserView::read(&mut context.pool(), local_user_id).await?;
  check_user_valid(&local_user_view.person)?;

  Ok(local_user_view)
}
