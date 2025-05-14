use super::convert_published_time;
use crate::community_use_pending;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection};
use lemmy_api_common::{
  build_response::{build_post_response, send_local_notifs},
  context::LemmyContext,
  plugins::{plugin_hook_after, plugin_hook_before},
  post::{CreatePost, PostResponse},
  request::{check_urls_are_images, generate_post_link_metadata},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_community_user_action, check_nsfw_allowed, get_url_blocklist, honeypot_check,
    process_post_urls, process_markdown_opt, send_webmention, slur_regex,
  },
};
use lemmy_db_schema::{
  impls::actor_language::validate_post_language,
  newtypes::PostOrCommentId,
  source::{
    community::Community,
    post::{Post, PostActions, PostInsertForm, PostLikeForm, PostReadForm},
    post_gallery::{PostGallery, PostGalleryInsertForm},
  },
  traits::{Crud, Likeable, Readable},
  utils::{diesel_url_create, get_conn},
};
use lemmy_db_views::structs::{CommunityModeratorView, LocalUserView, SiteView};
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType, LemmyResult},
  utils::{
    mention::scrape_text_for_mentions,
    slurs::check_slurs,
    validation::{is_valid_body_field, is_valid_post_title, is_valid_url},
  },
};

pub async fn create_post(
  data: Json<CreatePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  honeypot_check(&data.honeypot)?;
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;

  let slur_regex = slur_regex(&context).await?;
  check_slurs(&data.name, &slur_regex)?;
  let url_blocklist = get_url_blocklist(&context).await?;

  let body = process_markdown_opt(&data.body, &slur_regex, &url_blocklist, &context).await?;
  let (url, gallery_forms) = process_post_urls(&data.url, &url_blocklist)?.unwrap_or_default();
  let custom_thumbnail = diesel_url_create(data.custom_thumbnail.as_deref())?;
  let is_gallery = gallery_forms.as_deref().is_some_and(|v| v.len() > 1);
  check_nsfw_allowed(data.nsfw, Some(&local_site))?;

  is_valid_post_title(&data.name)?;

  if let Some(custom_thumbnail) = &custom_thumbnail {
    is_valid_url(custom_thumbnail)?;
  }

  if let Some(body) = &body {
    is_valid_body_field(body, true)?;
  }

  let community = Community::read(&mut context.pool(), data.community_id).await?;
  let community_id = community.id;
  check_community_user_action(&local_user_view, &community, &mut context.pool()).await?;

  // If its an NSFW community, then use that as a default
  let nsfw = data.nsfw.or(Some(community.nsfw));

  if community.posting_restricted_to_mods {
    let community_id = data.community_id;
    CommunityModeratorView::check_is_community_moderator(
      &mut context.pool(),
      community_id,
      local_user_view.local_user.person_id,
    )
    .await?;
  }

  let language_id = validate_post_language(
    &mut context.pool(),
    data.language_id,
    data.community_id,
    local_user_view.local_user.id,
  )
  .await?;
  let scheduled_publish_time =
    convert_published_time(data.scheduled_publish_time, &local_user_view, &context).await?;

  let mut post_form = PostInsertForm {
    body,
    nsfw,
    language_id: Some(language_id),
    federation_pending: Some(community_use_pending(&community, &context).await),
    scheduled_publish_time,
    ..PostInsertForm::new(
      data.name.trim().to_string(),
      local_user_view.person.id,
      data.community_id,
    )
  };

  let inserted_post = if let (Some(gallery_forms), true) = (&gallery_forms, is_gallery) {
    let gallery_forms = check_urls_are_images(gallery_forms, &context).await?;
    let (url, url_content_type) = gallery_forms
      .get(0)
      .map(|f| (Some(f.url.clone()), f.url_content_type.clone()))
      .unwrap_or_default();

    post_form.url = url;
    post_form.url_content_type = url_content_type;

    post_form = plugin_hook_before("before_create_local_post", post_form).await?;

    let pool = &mut context.pool();
    let conn = &mut get_conn(pool).await?;
    let inserted_post = conn
      .transaction::<_, LemmyError, _>(|conn| {
        async move {
          let post = Post::create(&mut conn.into(), &post_form)
            .await
            .with_lemmy_type(LemmyErrorType::CouldntCreatePost)?;

          let post_id = post.id;
          let gallert_forms = gallery_forms
            .iter()
            .map(|f| PostGalleryInsertForm {
              post_id: post_id,
              ..f.clone()
            })
            .collect::<Vec<_>>();

          PostGallery::create_from_vec(&gallert_forms, &mut conn.into())
            .await
            .with_lemmy_type(LemmyErrorType::CouldntCreatePost)?;

          Ok(post)
        }
        .scope_boxed()
      })
      .await?;

    plugin_hook_after("after_create_local_post", &inserted_post)?;

    if scheduled_publish_time.is_none() {
      ActivityChannel::submit_activity(
        SendActivityData::CreatePost(inserted_post.clone()),
        &context,
      )?;
    }

    inserted_post
  } else {
    // If there's only one gallery item, treat it like a posts with just a bare url.
    let (url, alt_text) = if url.is_none() {
      gallery_forms
        .and_then(|g| g.get(0).map(|g| (Some(g.url.clone()), g.alt_text.clone())))
        .unwrap_or_default()
    } else {
      (url, None)
    };

    post_form.url = url;

    post_form = plugin_hook_before("before_create_local_post", post_form).await?;

    let inserted_post = Post::create(&mut context.pool(), &post_form)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreatePost)?;

    plugin_hook_after("after_create_local_post", &inserted_post)?;

    let federate_post = if scheduled_publish_time.is_none() {
      send_webmention(inserted_post.clone(), community);
      |post| Some(SendActivityData::CreatePost(post))
    } else {
      |_| None
    };
    generate_post_link_metadata(
      inserted_post.clone(),
      custom_thumbnail.map(Into::into),
      alt_text,
      federate_post,
      context.reset_request_count(),
    )
    .await?;

    inserted_post
  };

  // They like their own post by default
  let person_id = local_user_view.person.id;
  let post_id = inserted_post.id;
  let local_instance_id = local_user_view.person.instance_id;
  let like_form = PostLikeForm::new(post_id, person_id, 1);

  PostActions::like(&mut context.pool(), &like_form).await?;

  // Scan the post body for user mentions, add those rows
  let mentions = scrape_text_for_mentions(&inserted_post.body.clone().unwrap_or_default());
  send_local_notifs(
    mentions,
    PostOrCommentId::Post(post_id),
    &local_user_view.person,
    true,
    &context,
    Some(&local_user_view),
    local_instance_id,
  )
  .await?;

  let read_form = PostReadForm::new(post_id, person_id);
  PostActions::mark_as_read(&mut context.pool(), &read_form).await?;

  build_post_response(&context, community_id, local_user_view, post_id).await
}
