use activitypub_federation::config::Data;
use actix_web::web::Json;
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
  source::{
    actor_language::CommunityLanguage,
    local_site::LocalSite,
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
  utils::{diesel_string_update, diesel_url_update, naive_now},
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  utils::{
    slurs::check_slurs,
    validation::{
      check_url_scheme,
      is_url_blocked,
      is_valid_alt_text_field,
      is_valid_body_field,
      is_valid_post_title,
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
    check_url_scheme(url)?;
  }

  if let Some(Some(custom_thumbnail)) = &custom_thumbnail {
    check_url_scheme(custom_thumbnail)?;
  }

  let post_id = data.post_id;
  let orig_post = Post::read(&mut context.pool(), post_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPost)?;

  check_community_user_action(
    &local_user_view.person,
    orig_post.community_id,
    &mut context.pool(),
  )
  .await?;

  // Verify that only the creator can edit
  if !Post::is_post_creator(local_user_view.person.id, orig_post.creator_id) {
    Err(LemmyErrorType::NoPostEditAllowed)?
  }

  let language_id = data.language_id;
  CommunityLanguage::is_allowed_community_language(
    &mut context.pool(),
    language_id,
    orig_post.community_id,
  )
  .await?;

  let post_form = PostUpdateForm {
    name: data.name.clone(),
    url,
    body,
    alt_text,
    nsfw: data.nsfw,
    language_id: data.language_id,
    updated: Some(Some(naive_now())),
    ..Default::default()
  };

  let post_id = data.post_id;
  let updated_post = Post::update(&mut context.pool(), post_id, &post_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdatePost)?;

  generate_post_link_metadata(
    updated_post.clone(),
    custom_thumbnail.flatten().map(Into::into),
    |post| Some(SendActivityData::UpdatePost(post)),
    Some(local_site),
    context.reset_request_count(),
  )
  .await?;

  build_post_response(
    context.deref(),
    orig_post.community_id,
    &local_user_view.person,
    post_id,
  )
  .await
}
