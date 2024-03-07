use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  site::SiteUrlBlocklist,
  utils::{get_url_blocklist, is_admin},
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn list_url_blocks(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<SiteUrlBlocklist>, LemmyError> {
  is_admin(&local_user_view)?;

  let urls = get_url_blocklist(&context).await?;

  Ok(Json(SiteUrlBlocklist { urls }))
}
