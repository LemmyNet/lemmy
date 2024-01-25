use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{EditPost, PostResponse},
  request::fetch_link_metadata,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_community_user_action,
    local_site_to_slur_regex,
    process_markdown_opt,
    proxy_image_link_opt_apub,
  },
};
use lemmy_db_schema::{
  source::{
    actor_language::CommunityLanguage,
    local_site::LocalSite,
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
  utils::{diesel_option_overwrite, naive_now},
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  utils::{
    slurs::check_slurs_opt,
    validation::{check_url_scheme, clean_url_params, is_valid_body_field, is_valid_post_title},
  },
};
use std::ops::Deref;

#[tracing::instrument(skip(context))]
pub async fn update_post(
  data: Json<EditPost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<PostResponse>, LemmyError> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  // TODO No good way to handle a clear.
  // Issue link: https://github.com/LemmyNet/lemmy/issues/2287
  let url = data.url.as_ref().map(clean_url_params);

  let slur_regex = local_site_to_slur_regex(&local_site);
  check_slurs_opt(&data.name, &slur_regex)?;
  let body = process_markdown_opt(&data.body, &slur_regex, &context).await?;

  if let Some(name) = &data.name {
    is_valid_post_title(name)?;
  }

  is_valid_body_field(&body, true)?;
  check_url_scheme(&data.url)?;

  let post_id = data.post_id;
  let orig_post = Post::read(&mut context.pool(), post_id).await?;

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

  // Fetch post links and Pictrs cached image if url was updated
  let (embed_title, embed_description, embed_video_url, thumbnail_url) = match &url {
    Some(url) => {
      let metadata = fetch_link_metadata(url, true, &context).await?;
      (
        Some(metadata.opengraph_data.title),
        Some(metadata.opengraph_data.description),
        Some(metadata.opengraph_data.embed_video_url),
        Some(metadata.thumbnail),
      )
    }
    _ => Default::default(),
  };
  let url = match url {
    Some(url) => Some(proxy_image_link_opt_apub(Some(url), &context).await?),
    _ => Default::default(),
  };

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
    body: diesel_option_overwrite(body),
    nsfw: data.nsfw,
    embed_title,
    embed_description,
    embed_video_url,
    language_id: data.language_id,
    thumbnail_url,
    updated: Some(Some(naive_now())),
    ..Default::default()
  };

  let post_id = data.post_id;
  let updated_post = Post::update(&mut context.pool(), post_id, &post_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdatePost)?;

  ActivityChannel::submit_activity(SendActivityData::UpdatePost(updated_post), &context).await?;

  build_post_response(
    context.deref(),
    orig_post.community_id,
    &local_user_view.person,
    post_id,
  )
  .await
}
