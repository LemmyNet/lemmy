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
use url::Url;

#[tracing::instrument(skip(context))]
pub async fn add_url_block(
  data: Json<SiteUrlBlock>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<SuccessResponse>, LemmyError> {
  is_admin(&local_user_view)?;

  // Make sure the URL is valid
  let url = Url::parse(&data.url)?;

  LocalSiteUrlBlocklist::add(&mut context.pool(), url.to_string()).await?;

  Ok(Json(SuccessResponse::default()))
}
