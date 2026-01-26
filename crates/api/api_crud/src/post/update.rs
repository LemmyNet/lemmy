use super::convert_published_time;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_utils::{
  build_response::build_post_response,
  context::LemmyContext,
  notify::NotifyData,
  plugins::{plugin_hook_after, plugin_hook_before},
  request::generate_post_link_metadata,
  send_activity::SendActivityData,
  utils::{
    check_community_user_action,
    check_nsfw_allowed,
    get_url_blocklist,
    process_markdown_opt,
    send_webmention,
    slur_regex,
    update_post_tags,
  },
};
use lemmy_db_schema::{
  impls::actor_language::validate_post_language,
  source::{
    community::Community,
    post::{Post, PostUpdateForm},
  },
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{
  PostView,
  api::{EditPost, PostResponse},
};
use lemmy_db_views_site::SiteView;
use lemmy_diesel_utils::{
  traits::Crud,
  utils::{diesel_string_update, diesel_url_update},
};
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
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

pub async fn edit_post(
  Json(data): Json<EditPost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  let local_instance_id = local_user_view.person.instance_id;
  let url = diesel_url_update(data.url.as_deref())?;

  let custom_thumbnail = diesel_url_update(data.custom_thumbnail.as_deref())?;

  let url_blocklist = get_url_blocklist(&context).await?;

  let slur_regex = slur_regex(&context).await?;

  let body = diesel_string_update(
    process_markdown_opt(&data.body, &slur_regex, &url_blocklist, &context)
      .await?
      .as_deref(),
  );

  check_nsfw_allowed(data.nsfw, Some(&local_site))?;

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
  let orig_post = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user_view.local_user),
    local_instance_id,
    false,
  )
  .await?;

  let nsfw = if orig_post.community.nsfw {
    Some(true)
  } else {
    data.nsfw
  };

  check_community_user_action(&local_user_view, &orig_post.community, &mut context.pool()).await?;

  // Verify that only the creator can edit
  if !Post::is_post_creator(local_user_view.person.id, orig_post.post.creator_id) {
    Err(LemmyErrorType::NoPostEditAllowed)?
  }

  // handle changes to scheduled_publish_time
  let scheduled_publish_time_at = match (
    orig_post.post.scheduled_publish_time_at,
    data.scheduled_publish_time_at,
  ) {
    // schedule time can be changed if post is still scheduled (and not published yet)
    (Some(_), Some(_)) => Some(
      convert_published_time(data.scheduled_publish_time_at, &local_user_view, &context).await?,
    ),
    // post was scheduled, gets changed to publish immediately
    (Some(_), None) => Some(None),
    // unchanged
    (_, _) => None,
  };

  let mut post_form = PostUpdateForm {
    name: data.name.clone(),
    url,
    body,
    alt_text,
    nsfw,
    language_id: data.language_id,
    updated_at: Some(Some(Utc::now())),
    scheduled_publish_time_at,
    ..Default::default()
  };
  post_form = plugin_hook_before("local_post_before_update", post_form).await?;
  validate_post_language(
    &mut context.pool(),
    post_form.language_id,
    orig_post.post.community_id,
  )
  .await?;

  let post_id = data.post_id;
  let updated_post = Post::update(&mut context.pool(), post_id, &post_form).await?;
  plugin_hook_after("local_post_after_update", &post_form);

  if let Some(tags) = &data.tags {
    update_post_tags(&orig_post.post, tags, &context).await?;
  }

  NotifyData::new(
    updated_post.clone(),
    local_user_view.person.clone(),
    orig_post.community.clone(),
  )
  .send(&context);

  // send out federation/webmention if necessary
  match (
    orig_post.post.scheduled_publish_time_at,
    data.scheduled_publish_time_at,
  ) {
    // schedule was removed, send create activity and webmention
    (Some(_), None) => {
      let community = Community::read(&mut context.pool(), orig_post.community.id).await?;
      send_webmention(updated_post.clone(), &community);
      generate_post_link_metadata(
        updated_post.clone(),
        custom_thumbnail.flatten().map(Into::into),
        |post| Some(SendActivityData::CreatePost(post)),
        context.clone(),
      )
      .await?;
    }
    // post was already public, send update
    (None, _) => {
      generate_post_link_metadata(
        updated_post.clone(),
        custom_thumbnail.flatten().map(Into::into),
        |post| Some(SendActivityData::UpdatePost(post)),
        context.clone(),
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
