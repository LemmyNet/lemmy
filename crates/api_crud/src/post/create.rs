use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{CreatePost, PostResponse},
  request::fetch_site_data,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_community_ban,
    check_community_deleted_or_removed,
    generate_local_apub_endpoint,
    honeypot_check,
    local_site_to_slur_regex,
    local_user_view_from_jwt,
    mark_post_as_read,
    sanitize_html,
    sanitize_html_opt,
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
};
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  spawn_try_task,
  utils::{
    slurs::{check_slurs, check_slurs_opt},
    validation::{check_url_scheme, clean_url_params, is_valid_body_field, is_valid_post_title},
  },
  SYNCHRONOUS_FEDERATION,
};
use tracing::Instrument;
use url::Url;
use webmention::{Webmention, WebmentionError};

#[tracing::instrument(skip(context))]
pub async fn create_post(
  data: Json<CreatePost>,
  context: Data<LemmyContext>,
) -> Result<Json<PostResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;
  let local_site = LocalSite::read(&mut context.pool()).await?;

  let slur_regex = local_site_to_slur_regex(&local_site);
  check_slurs(&data.name, &slur_regex)?;
  check_slurs_opt(&data.body, &slur_regex)?;
  honeypot_check(&data.honeypot)?;

  let data_url = data.url.as_ref();
  let url = data_url.map(clean_url_params).map(Into::into); // TODO no good way to handle a "clear"

  is_valid_post_title(&data.name)?;
  is_valid_body_field(&data.body, true)?;
  check_url_scheme(&data.url)?;

  check_community_ban(
    local_user_view.person.id,
    data.community_id,
    &mut context.pool(),
  )
  .await?;
  check_community_deleted_or_removed(data.community_id, &mut context.pool()).await?;

  let community_id = data.community_id;
  let community = Community::read(&mut context.pool(), community_id).await?;
  if community.posting_restricted_to_mods {
    let community_id = data.community_id;
    let is_mod = CommunityView::is_mod_or_admin(
      &mut context.pool(),
      local_user_view.local_user.person_id,
      community_id,
    )
    .await?;
    if !is_mod {
      return Err(LemmyErrorType::OnlyModsCanPostInCommunity)?;
    }
  }

  if community.posting_restricted_to_local {
    if !local_user_view.person.local {
      return Err(LemmyErrorType::OnlyLocalCanPostInCommunity)?;
    }
  }

  // Fetch post links and pictrs cached image
  let (metadata_res, thumbnail_url) =
    fetch_site_data(context.client(), context.settings(), data_url, true).await;
  let (embed_title, embed_description, embed_video_url) = metadata_res
    .map(|u| (u.title, u.description, u.embed_video_url))
    .unwrap_or_default();

  let name = sanitize_html(data.name.trim());
  let body = sanitize_html_opt(&data.body);
  let embed_title = sanitize_html_opt(&embed_title);
  let embed_description = sanitize_html_opt(&embed_description);

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
    .name(name)
    .url(url)
    .body(body)
    .community_id(data.community_id)
    .creator_id(local_user_view.person.id)
    .nsfw(data.nsfw)
    .embed_title(embed_title)
    .embed_description(embed_description)
    .embed_video_url(embed_video_url)
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
    let task = async move {
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
    };
    if *SYNCHRONOUS_FEDERATION {
      task.await?;
    } else {
      spawn_try_task(task);
    }
  };

  build_post_response(&context, community_id, person_id, post_id).await
}
