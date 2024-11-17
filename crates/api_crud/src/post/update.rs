use super::{convert_published_time, create::send_webmention};
use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{EditPost, PostResponse},
  request::generate_post_link_metadata,
  send_activity::SendActivityData,
  utils::{
    check_community_user_action,
    get_url_blocklist,
    local_site_to_slur_regex,
    process_markdown_opt,
  },
};
use lemmy_db_schema::{
  impls::actor_language::validate_post_language,
  source::{
    community::Community,
    local_site::LocalSite,
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
  utils::{diesel_string_update, diesel_url_update},
};
use lemmy_db_views::structs::{LocalUserView, PostView};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  utils::{
    slurs::check_slurs,
    validation::{
      is_url_blocked,
      is_valid_alt_text_field,
      is_valid_body_field,
      is_valid_post_title,
      is_valid_url,
    },
  },
};
use std::ops::Deref;

#[tracing::instrument(skip(context))]
pub async fn update_post(
  data: Json<EditPost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  let url = diesel_url_update(data.url.as_deref())?;

  let custom_thumbnail = diesel_url_update(data.custom_thumbnail.as_deref())?;

  let url_blocklist = get_url_blocklist(&context).await?;

  let slur_regex = local_site_to_slur_regex(&local_site);

  let body = diesel_string_update(
    process_markdown_opt(&data.body, &slur_regex, &url_blocklist, &context)
      .await?
      .as_deref(),
  );

  let alt_text = diesel_string_update(data.alt_text.as_deref());

  if let Some(name) = &data.name {
    is_valid_post_title(name)?;
    check_slurs(name, &slur_regex)?;
  }

  if let Some(Some(body)) = &body {
    is_valid_body_field(body, true)?;
  }

  if let Some(Some(alt_text)) = &alt_text {
    is_valid_alt_text_field(alt_text)?;
  }

  if let Some(Some(url)) = &url {
    is_url_blocked(url, &url_blocklist)?;
    is_valid_url(url)?;
  }

  if let Some(Some(custom_thumbnail)) = &custom_thumbnail {
    is_valid_url(custom_thumbnail)?;
  }

  let post_id = data.post_id;
  let orig_post = PostView::read(&mut context.pool(), post_id, None, false).await?;

  check_community_user_action(
    &local_user_view.person,
    &orig_post.community,
    &mut context.pool(),
  )
  .await?;

  // Verify that only the creator can edit
  if !Post::is_post_creator(local_user_view.person.id, orig_post.post.creator_id) {
    Err(LemmyErrorType::NoPostEditAllowed)?
  }

  let language_id = validate_post_language(
    &mut context.pool(),
    data.language_id,
    orig_post.post.community_id,
    local_user_view.local_user.id,
  )
  .await?;

  // handle changes to scheduled_publish_time
  let scheduled_publish_time = match (
    orig_post.post.scheduled_publish_time,
    data.scheduled_publish_time,
  ) {
    // schedule time can be changed if post is still scheduled (and not published yet)
    (Some(_), Some(_)) => {
      Some(convert_published_time(data.scheduled_publish_time, &local_user_view, &context).await?)
    }
    // post was scheduled, gets changed to publish immediately
    (Some(_), None) => Some(None),
    // unchanged
    (_, _) => None,
  };

  let post_form = PostUpdateForm {
    name: data.name.clone(),
    url,
    body,
    alt_text,
    nsfw: data.nsfw,
    language_id: Some(language_id),
    updated: Some(Some(Utc::now())),
    scheduled_publish_time,
    ..Default::default()
  };

  let post_id = data.post_id;
  let updated_post = Post::update(&mut context.pool(), post_id, &post_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdatePost)?;

  // send out federation/webmention if necessary
  match (
    orig_post.post.scheduled_publish_time,
    data.scheduled_publish_time,
  ) {
    // schedule was removed, send create activity and webmention
    (Some(_), None) => {
      let community = Community::read(&mut context.pool(), orig_post.community.id).await?;
      send_webmention(updated_post.clone(), community);
      generate_post_link_metadata(
        updated_post.clone(),
        custom_thumbnail.flatten().map(Into::into),
        |post| Some(SendActivityData::CreatePost(post)),
        context.reset_request_count(),
      )
      .await?;
    }
    // post was already public, send update
    (None, _) => {
      generate_post_link_metadata(
        updated_post.clone(),
        custom_thumbnail.flatten().map(Into::into),
        |post| Some(SendActivityData::UpdatePost(post)),
        context.reset_request_count(),
      )
      .await?
    }
    // schedule was changed, do nothing
    (Some(_), Some(_)) => {}
  };

  build_post_response(
    context.deref(),
    orig_post.community.id,
    local_user_view,
    post_id,
  )
  .await
}
