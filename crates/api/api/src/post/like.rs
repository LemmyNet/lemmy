use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  build_response::build_post_response,
  context::LemmyContext,
  plugins::{plugin_hook_after, plugin_hook_before},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_bot_account,
    check_community_user_action,
    check_local_user_valid,
    check_local_vote_mode,
  },
};
use lemmy_db_schema::{
  newtypes::PostOrCommentId,
  source::{
    notification::Notification,
    person::PersonActions,
    post::{PostActions, PostLikeForm},
  },
  traits::Likeable,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{
  PostView,
  api::{CreatePostLike, PostResponse},
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;
use std::ops::Deref;

pub async fn like_post(
  data: Json<CreatePostLike>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  check_local_user_valid(&local_user_view)?;
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  let local_instance_id = local_user_view.person.instance_id;
  let post_id = data.post_id;
  let my_person_id = local_user_view.person.id;

  check_local_vote_mode(
    data.is_upvote,
    PostOrCommentId::Post(post_id),
    &local_site,
    my_person_id,
    &mut context.pool(),
  )
  .await?;
  check_bot_account(&local_user_view.person)?;

  // Check for a community ban
  let orig_post = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user_view.local_user),
    local_instance_id,
    false,
  )
  .await?;
  let previous_is_upvote = orig_post.post_actions.and_then(|p| p.vote_is_upvote);

  check_community_user_action(&local_user_view, &orig_post.community, &mut context.pool()).await?;

  // Remove any likes first
  PostActions::remove_like(&mut context.pool(), my_person_id, post_id).await?;
  if let Some(previous_is_upvote) = previous_is_upvote {
    PersonActions::remove_like(
      &mut context.pool(),
      my_person_id,
      orig_post.creator.id,
      previous_is_upvote,
    )
    .await
    // Ignore errors, since a previous_like of zero throws an error
    .ok();
  }

  if let Some(is_upvote) = data.is_upvote {
    let mut like_form = PostLikeForm::new(data.post_id, my_person_id, is_upvote);
    like_form = plugin_hook_before("post_before_vote", like_form).await?;
    let like = PostActions::like(&mut context.pool(), &like_form).await?;
    PersonActions::like(
      &mut context.pool(),
      my_person_id,
      orig_post.creator.id,
      like_form.vote_is_upvote,
    )
    .await?;

    plugin_hook_after("post_after_vote", &like);
  }

  // Mark Post Read
  PostActions::mark_as_read(&mut context.pool(), my_person_id, &[post_id]).await?;

  // Mark any notifications as read
  Notification::mark_read_by_post_and_recipient(&mut context.pool(), post_id, my_person_id, true)
    .await
    .ok();

  ActivityChannel::submit_activity(
    SendActivityData::LikePostOrComment {
      object_id: orig_post.post.ap_id,
      actor: local_user_view.person.clone(),
      community: orig_post.community.clone(),
      previous_is_upvote,
      new_is_upvote: data.is_upvote,
    },
    &context,
  )?;

  build_post_response(
    context.deref(),
    orig_post.community.id,
    local_user_view,
    post_id,
  )
  .await
}
