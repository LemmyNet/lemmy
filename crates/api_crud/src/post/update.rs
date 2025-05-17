use super::convert_published_time;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::Utc;
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection};
use lemmy_api_common::{
  build_response::{build_post_response, send_local_notifs},
  context::LemmyContext,
  plugins::{plugin_hook_after, plugin_hook_before},
  post::{EditPost, PostResponse},
  request::generate_post_link_metadata,
  send_activity::SendActivityData,
  tags::update_post_tags,
  utils::{
    check_community_user_action,
    check_nsfw_allowed,
    get_url_blocklist,
    process_gallery,
    process_markdown_opt,
    send_webmention,
    slur_regex,
  },
};
use lemmy_db_schema::{
  impls::actor_language::validate_post_language,
  newtypes::PostOrCommentId,
  source::{
    community::Community,
    post::{Post, PostUpdateForm},
    post_gallery::{PostGallery, PostGalleryInsertForm},
  },
  traits::Crud,
  utils::{diesel_string_update, diesel_url_update, get_conn},
};
use lemmy_db_views_community::CommunityView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::PostView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorType, LemmyResult},
  utils::{
    mention::scrape_text_for_mentions,
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

pub async fn update_post(
  data: Json<EditPost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  let local_instance_id = local_user_view.person.instance_id;

  let custom_thumbnail = diesel_url_update(data.custom_thumbnail.as_deref())?;

  let url_blocklist = get_url_blocklist(&context).await?;
  let url = diesel_url_update(data.url.as_deref())?;
  let gallery_forms = process_gallery(data.gallery.as_ref(), &url_blocklist)?;
  let alt_text = diesel_string_update(data.alt_text.as_deref());
  let slur_regex = slur_regex(&context).await?;

  let body = diesel_string_update(
    process_markdown_opt(&data.body, &slur_regex, &url_blocklist, &context)
      .await?
      .as_deref(),
  );

  check_nsfw_allowed(data.nsfw, Some(&local_site))?;

  if let Some(Some(url)) = &url {
    is_url_blocked(url, &url_blocklist)?;
    is_valid_url(url)?;
  }

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

  if let Some(Some(custom_thumbnail)) = &custom_thumbnail {
    is_valid_url(custom_thumbnail)?;
  }

  let post_id = data.post_id;
  let orig_post =
    PostView::read(&mut context.pool(), post_id, None, local_instance_id, false).await?;
  let orig_gallery = PostGallery::list_from_post_id(post_id, &mut context.pool()).await?;

  check_community_user_action(&local_user_view, &orig_post.community, &mut context.pool()).await?;

  if let Some(tags) = &data.tags {
    // post view does not include communityview.post_tags
    let community_view =
      CommunityView::read(&mut context.pool(), orig_post.community.id, None, false).await?;
    update_post_tags(
      &context,
      &orig_post.post,
      &community_view,
      tags,
      &local_user_view,
    )
    .await?;
  }

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

  let mut post_form = PostUpdateForm {
    url,
    name: data.name.clone(),
    body,
    nsfw: data.nsfw,
    language_id: Some(language_id),
    updated: Some(Some(Utc::now())),
    scheduled_publish_time,
    ..Default::default()
  };

  if !orig_gallery.is_empty() && gallery_forms.is_none() {
    PostGallery::delete_from_post_id(post_id, &mut context.pool()).await?;
  }

  post_form = plugin_hook_before("before_update_local_post", post_form).await?;

  let post_id = data.post_id;
  // let updated_post = Post::update(&mut context.pool(), post_id, &post_form)
  //   .await
  //   .with_lemmy_type(LemmyErrorType::CouldntUpdatePost)?;
  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  let updated_post = conn
    .transaction::<_, LemmyError, _>(|conn| {
      async move {
        let post = Post::update(&mut conn.into(), post_id, &post_form).await?;

        if let Some(gallery_forms) = gallery_forms {
          let gallert_forms = gallery_forms
            .iter()
            .map(|f| PostGalleryInsertForm {
              post_id,
              ..f.clone()
            })
            .collect::<Vec<_>>();

          PostGallery::create_from_vec(&gallert_forms, &mut conn.into()).await?;
        }

        Ok(post)
      }
      .scope_boxed()
    })
    .await?;

  plugin_hook_after("after_update_local_post", &updated_post.clone())?;

  // Scan the post body for user mentions, add those rows
  let mentions = scrape_text_for_mentions(&updated_post.body.clone().unwrap_or_default());
  send_local_notifs(
    mentions,
    PostOrCommentId::Post(updated_post.id),
    &local_user_view.person,
    false,
    &context,
    Some(&local_user_view),
    local_instance_id,
  )
  .await?;

  // send out federation/webmention if necessary
  match (
    orig_post.post.scheduled_publish_time,
    data.scheduled_publish_time,
  ) {
    // schedule was removed, send create activity and webmention
    (Some(_), None) => {
      let community = Community::read(&mut context.pool(), orig_post.community.id).await?;
      send_webmention(updated_post.clone(), &community);
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
