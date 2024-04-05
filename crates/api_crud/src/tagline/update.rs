use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  tagline::{TaglineResponse, UpdateTagline},
  utils::{get_url_blocklist, is_admin, local_site_to_slur_regex, process_markdown_opt},
};
use lemmy_db_schema::{
  source::{
    local_site::LocalSite,
    tagline::{Tagline, TaglineUpdateForm},
  },
  traits::Crud,
  utils::naive_now,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{error::LemmyError, utils::validation::is_valid_body_field};

#[tracing::instrument(skip(context))]
pub async fn update_tagline(
  data: Json<UpdateTagline>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<TaglineResponse>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let local_site = LocalSite::read(&mut context.pool()).await?;

  let slur_regex = local_site_to_slur_regex(&local_site);
  let url_blocklist = get_url_blocklist(&context).await?;
  let content = process_markdown_opt(&data.content, &slur_regex, &url_blocklist, &context).await?;
  is_valid_body_field(&content, false)?;

  let tagline_form = TaglineUpdateForm {
    content,
    updated: Some(Some(naive_now())),
  };

  let tagline = Tagline::update(&mut context.pool(), data.id, &tagline_form).await?;

  Ok(Json(TaglineResponse { tagline }))
}
