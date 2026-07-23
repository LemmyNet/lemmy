use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  build_response::build_comment_response,
  context::LemmyContext,
  plugins::{plugin_hook_after, plugin_hook_before},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_bot_account,
    check_community_user_action,
    check_local_user_banned_or_deleted,
    check_vote_settings,
  },
};
use lemmy_db_schema::{
  newtypes::PostOrCommentId,
  source::{
    comment::{CommentActions, CommentLikeForm},
    notification::Notification,
    person::PersonActions,
  },
  traits::Likeable,
};
use lemmy_db_views_comment::{
  CommentView,
  api::{CommentResponse, CreateCommentLike},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;
use std::ops::Deref;

pub async fn like_comment(
  Json(data): Json<CreateCommentLike>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponse>> {
  check_local_user_banned_or_deleted(&local_user_view)?;
  let local_instance_id = local_user_view.person.instance_id;
  let comment_id = data.comment_id;
  let my_person_id = local_user_view.person.id;

  let orig_comment = CommentView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  check_vote_settings(
    data.is_upvote,
    PostOrCommentId::Comment(comment_id),
    &orig_comment.community,
    &local_user_view.person,
    &context,
  )
  .await?;
  check_bot_account(&local_user_view.person)?;

  let previous_is_upvote = orig_comment.comment_actions.and_then(|p| p.vote_is_upvote);

  check_community_user_action(
    &local_user_view,
    &orig_comment.community,
    &mut context.pool(),
  )
  .await?;

  let mut like_form = CommentLikeForm::new(data.comment_id, my_person_id, data.is_upvote);
  like_form = plugin_hook_before("comment_before_vote", like_form).await?;
  let like = CommentActions::like(&mut context.pool(), &like_form).await?;
  PersonActions::like(
    &mut context.pool(),
    my_person_id,
    orig_comment.creator.id,
    previous_is_upvote,
    data.is_upvote,
  )
  .await?;

  plugin_hook_after("comment_after_vote", &like);

  // Mark any notification as read
  Notification::mark_read_by_comment_and_recipient(
    &mut context.pool(),
    comment_id,
    my_person_id,
    true,
  )
  .await
  .ok();

  ActivityChannel::submit_activity(
    SendActivityData::LikePostOrComment {
      object_id: orig_comment.comment.ap_id,
      actor: local_user_view.person.clone(),
      community: orig_comment.community,
      previous_is_upvote,
      new_is_upvote: data.is_upvote,
    },
    &context,
  )?;

  Ok(Json(
    build_comment_response(
      context.deref(),
      comment_id,
      Some(local_user_view),
      local_instance_id,
    )
    .await?,
  ))
}
