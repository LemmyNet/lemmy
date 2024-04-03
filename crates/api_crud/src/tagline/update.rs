use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  tagline::{TaglineResponse, UpdateTagline},
  utils::is_admin,
};
use lemmy_db_schema::{
  source::{
    local_site::LocalSite,
    tagline::{Tagline, TaglineUpdateForm},
  },
  utils::naive_now,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn update_tagline(
  data: Json<UpdateTagline>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<TaglineResponse>, LemmyError> {
  let local_site = LocalSite::read(&mut context.pool()).await?;
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let tagline_form = TaglineUpdateForm {
    local_site_id: local_site.id,
    content: data.content.to_string(),
    updated: Some(naive_now()),
  };

  let tagline = Tagline::update(&mut context.pool(), data.id, &tagline_form).await?;

  Ok(Json(TaglineResponse { tagline }))
}
