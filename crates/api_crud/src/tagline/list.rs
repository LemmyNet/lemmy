use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  tagline::{ListTaglines, ListTaglinesResponse},
};
use lemmy_db_views::structs::{TaglineView, LocalUserView, SiteView};
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn list_taglines(
  data: Query<ListTaglines>,
  local_user_view: Option<LocalUserView>,
  context: Data<LemmyContext>,
) -> Result<Json<ListTaglinesResponse>, LemmyError> {

  let local_site = SiteView::read_local(&mut context.pool()).await?;
  let taglines = TaglineView::list(
    &mut context.pool(),
    local_site.local_site.id,
    data.page,
    data.limit,
  )
  .await    
  .map_err(|e| anyhow::anyhow!("Failed to construct taglines response: {e}"))?;

  Ok(Json(ListTaglinesResponse { taglines }))
}
