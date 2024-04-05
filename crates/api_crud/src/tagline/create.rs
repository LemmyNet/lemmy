use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  tagline::{CreateTagline, TaglineResponse},
  utils::is_admin,
};
use lemmy_db_schema::{
  source::{
    local_site::LocalSite,
    tagline::{Tagline, TaglineInsertForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn create_tagline(
  data: Json<CreateTagline>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<TaglineResponse>, LemmyError> {
  let local_site = LocalSite::read(&mut context.pool()).await?;
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let tagline_form = TaglineInsertForm {
    local_site_id: local_site.id,
    content: data.content.to_string(),
  };

  let tagline = Tagline::create(&mut context.pool(), &tagline_form).await?;

  Ok(Json(TaglineResponse { tagline }))
}
