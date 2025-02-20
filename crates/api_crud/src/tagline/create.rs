use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  tagline::{CreateTagline, TaglineResponse},
  utils::{get_url_blocklist, is_admin, process_markdown, slur_regex},
};
use lemmy_db_schema::{
  source::tagline::{Tagline, TaglineInsertForm},
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;

pub async fn create_tagline(
  data: Json<CreateTagline>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<TaglineResponse>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  let content = process_markdown(&data.content, &slur_regex, &url_blocklist, &context).await?;

  let tagline_form = TaglineInsertForm { content };

  let tagline = Tagline::create(&mut context.pool(), &tagline_form).await?;

  Ok(Json(TaglineResponse { tagline }))
}
