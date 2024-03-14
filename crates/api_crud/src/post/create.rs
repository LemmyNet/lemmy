use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{CreatePost, PostResponse},
  request::fetch_link_metadata_opt,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_community_user_action,
    generate_local_apub_endpoint,
    honeypot_check,
    local_site_to_slur_regex,
    mark_post_as_read,
    process_markdown_opt,
    proxy_image_link_opt_apub,
    EndpointType,
  },
};
use lemmy_db_schema::{
  impls::actor_language::default_post_language,
  source::{
    actor_language::CommunityLanguage,
    community::Community,
    local_site::LocalSite,
    post::{Post, PostInsertForm, PostLike, PostLikeForm, PostUpdateForm},
  },
  traits::{Crud, Likeable},
  CommunityVisibility,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::CommunityModeratorView;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  spawn_try_task,
  utils::{
    slurs::check_slurs,
    validation::{
      check_url_scheme,
      clean_url_params,
      is_valid_alt_text_field,
      is_valid_body_field,
      is_valid_post_title,
    },
  },
};
use tracing::Instrument;
use url::Url;
use webmention::{Webmention, WebmentionError};

#[tracing::instrument(skip(context))]
pub async fn create_post(
  data: Json<CreatePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<PostResponse>, LemmyError> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  honeypot_check(&data.honeypot)?;

  let slur_regex = local_site_to_slur_regex(&local_site);
  check_slurs(&data.name, &slur_regex)?;

  let body = process_markdown_opt(&data.body, &slur_regex, &context).await?;
  let data_url = data.url.as_ref();
  let url = data_url.map(clean_url_params); // TODO no good way to handle a "clear"
  let custom_thumbnail = data.custom_thumbnail.as_ref().map(clean_url_params);

  is_valid_post_title(&data.name)?;
  is_valid_body_field(&body, true)?;
  is_valid_alt_text_field(&data.alt_text)?;
  check_url_scheme(&url)?;
  check_url_scheme(&custom_thumbnail)?;

  check_community_user_action(
    &local_user_view.person,
    data.community_id,
    &mut context.pool(),
  )
  .await?;

  let community_id = data.community_id;
  let community = Community::read(&mut context.pool(), community_id).await?;
  if community.posting_restricted_to_mods {
    let community_id = data.community_id;
    let is_mod = CommunityModeratorView::is_community_moderator(
      &mut context.pool(),
      community_id,
      local_user_view.local_user.person_id,
    )
    .await?;
    if !is_mod {
      Err(LemmyErrorType::OnlyModsCanPostInCommunity)?
    }
  }

  // Only generate the thumbnail if there's no custom thumbnail provided,
  // otherwise it will save it in pictrs
  let generate_thumbnail = custom_thumbnail.is_none();

  // Fetch post links and pictrs cached image
  let metadata = fetch_link_metadata_opt(url.as_ref(), generate_thumbnail, &context).await;
  let url = proxy_image_link_opt_apub(url, &context).await?;
  let thumbnail_url = proxy_image_link_opt_apub(custom_thumbnail, &context)
    .await?
    .map(Into::into)
    .or(metadata.thumbnail);

  // Only need to check if language is allowed in case user set it explicitly. When using default
  // language, it already only returns allowed languages.
  CommunityLanguage::is_allowed_community_language(
    &mut context.pool(),
    data.language_id,
    community_id,
  )
  .await?;

  // attempt to set default language if none was provided
  let language_id = match data.language_id {
    Some(lid) => Some(lid),
    None => {
      default_post_language(
        &mut context.pool(),
        community_id,
        local_user_view.local_user.id,
      )
      .await?
    }
  };

  let post_form = PostInsertForm::builder()
    .name(data.name.trim().to_string())
    .url_content_type(metadata.content_type)
    .url(url)
    .body(body)
    .alt_text(data.alt_text.clone())
    .community_id(data.community_id)
    .creator_id(local_user_view.person.id)
    .nsfw(data.nsfw)
    .embed_title(metadata.opengraph_data.title)
    .embed_description(metadata.opengraph_data.description)
    .embed_video_url(metadata.opengraph_data.embed_video_url)
    .language_id(language_id)
    .thumbnail_url(thumbnail_url)
    .build();

  let inserted_post = Post::create(&mut context.pool(), &post_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntCreatePost)?;

  let inserted_post_id = inserted_post.id;
  let protocol_and_hostname = context.settings().get_protocol_and_hostname();
  let apub_id = generate_local_apub_endpoint(
    EndpointType::Post,
    &inserted_post_id.to_string(),
    &protocol_and_hostname,
  )?;
  let updated_post = Post::update(
    &mut context.pool(),
    inserted_post_id,
    &PostUpdateForm {
      ap_id: Some(apub_id),
      ..Default::default()
    },
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntCreatePost)?;

  // They like their own post by default
  let person_id = local_user_view.person.id;
  let post_id = inserted_post.id;
  let like_form = PostLikeForm {
    post_id,
    person_id,
    score: 1,
  };

  PostLike::like(&mut context.pool(), &like_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntLikePost)?;

  ActivityChannel::submit_activity(SendActivityData::CreatePost(updated_post.clone()), &context)
    .await?;

  // Mark the post as read
  mark_post_as_read(person_id, post_id, &mut context.pool()).await?;

  if let Some(url) = updated_post.url.clone() {
    if community.visibility == CommunityVisibility::Public {
      spawn_try_task(async move {
        let mut webmention =
          Webmention::new::<Url>(updated_post.ap_id.clone().into(), url.clone().into())?;
        webmention.set_checked(true);
        match webmention
          .send()
          .instrument(tracing::info_span!("Sending webmention"))
          .await
        {
          Err(WebmentionError::NoEndpointDiscovered(_)) => Ok(()),
          Ok(_) => Ok(()),
          Err(e) => Err(e).with_lemmy_type(LemmyErrorType::CouldntSendWebmention),
        }
      });
    }
  };

  build_post_response(&context, community_id, &local_user_view.person, post_id).await
}
