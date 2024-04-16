use lemmy_api_common::{claims::Claims, context::LemmyContext, utils::check_user_valid};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

pub mod feeds;
pub mod images;
pub mod nodeinfo;
pub mod webfinger;

#[tracing::instrument(skip_all)]
async fn local_user_view_from_jwt(jwt: &str, context: &LemmyContext) -> LemmyResult<LocalUserView> {
  let local_user_id = Claims::validate(jwt, context).await?;
  let local_user_view = LocalUserView::read(&mut context.pool(), local_user_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindLocalUser)?;
  check_user_valid(&local_user_view.person)?;

  Ok(local_user_view)
}
