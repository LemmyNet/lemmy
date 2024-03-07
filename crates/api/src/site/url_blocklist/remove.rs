use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  site::SiteUrlBlock,
  utils::is_admin,
  SuccessResponse,
};
use lemmy_db_schema::source::local_site_url_blocklist::LocalSiteUrlBlocklist;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn remove_url_block(
  data: Json<SiteUrlBlock>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<SuccessResponse>, LemmyError> {
  is_admin(&local_user_view)?;

  LocalSiteUrlBlocklist::remove(&mut context.pool(), data.url.clone()).await?;

  Ok(Json(SuccessResponse::default()))
}
