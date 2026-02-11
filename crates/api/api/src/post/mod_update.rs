use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_utils::{
  build_response::build_post_response,
  context::LemmyContext,
  plugins::{plugin_hook_after, plugin_hook_before},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_community_user_action,
    check_is_mod_or_admin,
    check_nsfw_allowed,
    update_post_tags,
  },
};
use lemmy_db_schema::source::post::{Post, PostUpdateForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{
  PostView,
  api::{ModEditPost, PostResponse},
};
use lemmy_db_views_site::SiteView;
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::LemmyResult;
use std::ops::Deref;

pub async fn mod_edit_post(
  Json(data): Json<ModEditPost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  let local_instance_id = local_user_view.person.instance_id;

  check_nsfw_allowed(data.nsfw, Some(&local_site))?;

  let post_id = data.post_id;
  let orig_post = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user_view.local_user),
    local_instance_id,
    false,
  )
  .await?;
  let community = orig_post.community;

  check_community_user_action(&local_user_view, &community, &mut context.pool()).await?;
  check_is_mod_or_admin(&mut context.pool(), local_user_view.person.id, community.id).await?;

  let mut post_form = PostUpdateForm {
    nsfw: data.nsfw,
    updated_at: Some(Some(Utc::now())),
    ..Default::default()
  };
  post_form = plugin_hook_before("local_post_before_vote", post_form).await?;

  let post_id = data.post_id;
  let updated_post = Post::update(&mut context.pool(), post_id, &post_form).await?;
  plugin_hook_after("local_post_after_vote", &post_form);

  if let Some(tags) = &data.tags {
    update_post_tags(&updated_post, tags, &context).await?;
  }

  ActivityChannel::submit_activity(SendActivityData::UpdatePost(updated_post.clone()), &context)?;

  build_post_response(context.deref(), community.id, local_user_view, post_id).await
}
