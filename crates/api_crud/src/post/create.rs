use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{CreatePost, PostResponse},
  request::generate_post_link_metadata,
  send_activity::SendActivityData,
  utils::{
    check_community_user_action,
    get_url_blocklist,
    honeypot_check,
    local_site_to_slur_regex,
    mark_post_as_read,
    process_markdown_opt,
  },
};
use lemmy_db_schema::{
  impls::actor_language::default_post_language,
  source::{
    actor_language::CommunityLanguage,
    community::Community,
    local_site::LocalSite,
    post::{Post, PostInsertForm, PostLike, PostLikeForm},
  },
  traits::{Crud, Likeable},
  utils::diesel_url_create,
  CommunityVisibility,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::CommunityModeratorView;
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  spawn_try_task,
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
use tracing::Instrument;
use url::Url;
use webmention::{Webmention, WebmentionError};

#[tracing::instrument(skip(context))]
pub async fn create_post(
  data: Json<CreatePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  honeypot_check(&data.honeypot)?;

  let slur_regex = local_site_to_slur_regex(&local_site);
  check_slurs(&data.name, &slur_regex)?;
  let url_blocklist = get_url_blocklist(&context).await?;

  let body = process_markdown_opt(&data.body, &slur_regex, &url_blocklist, &context).await?;
  let url = diesel_url_create(data.url.as_deref())?;
  let custom_thumbnail = diesel_url_create(data.custom_thumbnail.as_deref())?;

  is_valid_post_title(&data.name)?;

  if let Some(url) = &url {
    is_url_blocked(url, &url_blocklist)?;
    is_valid_url(url)?;
  }

  if let Some(custom_thumbnail) = &custom_thumbnail {
    is_valid_url(custom_thumbnail)?;
  }

  if let Some(alt_text) = &data.alt_text {
    is_valid_alt_text_field(alt_text)?;
  }

  if let Some(body) = &body {
    is_valid_body_field(body, true)?;
  }

  check_community_user_action(
    &local_user_view.person,
    data.community_id,
    &mut context.pool(),
  )
  .await?;

  let community_id = data.community_id;
  let community = Community::read(&mut context.pool(), community_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindCommunity)?;
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
    .url(url.map(Into::into))
    .body(body)
    .alt_text(data.alt_text.clone())
    .community_id(data.community_id)
    .creator_id(local_user_view.person.id)
    .nsfw(data.nsfw)
    .language_id(language_id)
    .build();

  let inserted_post = Post::create(&mut context.pool(), &post_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntCreatePost)?;

  generate_post_link_metadata(
    inserted_post.clone(),
    custom_thumbnail.map(Into::into),
    |post| Some(SendActivityData::CreatePost(post)),
    context.reset_request_count(),
  )
  .await?;

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

  mark_post_as_read(person_id, post_id, &mut context.pool()).await?;

  if let Some(url) = inserted_post.url.clone() {
    if community.visibility == CommunityVisibility::Public {
      spawn_try_task(async move {
        let mut webmention =
          Webmention::new::<Url>(inserted_post.ap_id.clone().into(), url.clone().into())?;
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

  build_post_response(&context, community_id, local_user_view, post_id).await
}
