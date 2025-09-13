use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  build_response::build_post_response,
  context::LemmyContext,
  plugins::{plugin_hook_after, plugin_hook_before},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_bot_account, check_community_user_action, check_local_vote_mode},
};
use lemmy_db_schema::{
  newtypes::PostOrCommentId,
  source::{
    person::PersonActions,
    post::{PostActions, PostLikeForm, PostReadForm},
  },
  traits::Likeable,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{
  api::{CreatePostLike, PostResponse},
  PostView,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;
use std::ops::Deref;

pub async fn like_post(
  data: Json<CreatePostLike>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  let local_instance_id = local_user_view.person.instance_id;
  let post_id = data.post_id;
  let my_person_id = local_user_view.person.id;

  check_local_vote_mode(
    data.score_is_positive,
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
  let previous_score_is_positive = orig_post
    .post_actions
    .and_then(|p| p.like_score_is_positive);

  check_community_user_action(&local_user_view, &orig_post.community, &mut context.pool()).await?;

  // Remove any likes first
  PostActions::remove_like(&mut context.pool(), my_person_id, post_id).await?;
  if let Some(previous_score_is_positive) = previous_score_is_positive {
    PersonActions::remove_like(
      &mut context.pool(),
      my_person_id,
      orig_post.creator.id,
      previous_score_is_positive,
    )
    .await
    // Ignore errors, since a previous_like of zero throws an error
    .ok();
  }

  if let Some(score_is_positive) = data.score_is_positive {
    let mut like_form = PostLikeForm::new(data.post_id, my_person_id, score_is_positive);
    like_form = plugin_hook_before("before_post_vote", like_form).await?;
    let like = PostActions::like(&mut context.pool(), &like_form).await?;
    PersonActions::like(
      &mut context.pool(),
      my_person_id,
      orig_post.creator.id,
      like_form.like_score_is_positive,
    )
    .await?;

    plugin_hook_after("after_post_vote", &like)?;
  }

  // Mark Post Read
  let read_form = PostReadForm::new(post_id, my_person_id);
  PostActions::mark_as_read(&mut context.pool(), &read_form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::LikePostOrComment {
      object_id: orig_post.post.ap_id,
      actor: local_user_view.person.clone(),
      community: orig_post.community.clone(),
      previous_score_is_positive,
      new_score_is_positive: data.score_is_positive,
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
