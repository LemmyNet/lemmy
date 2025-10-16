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
    check_local_user_valid,
    check_local_vote_mode,
  },
};
use lemmy_db_schema::{
  newtypes::PostOrCommentId,
  source::{
    comment::{CommentActions, CommentLikeForm},
    person::PersonActions,
  },
  traits::Likeable,
};
use lemmy_db_views_comment::{
  api::{CommentResponse, CreateCommentLike},
  CommentView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;
use std::ops::Deref;

pub async fn like_comment(
  data: Json<CreateCommentLike>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponse>> {
  check_local_user_valid(&local_user_view)?;
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  let local_instance_id = local_user_view.person.instance_id;
  let comment_id = data.comment_id;
  let my_person_id = local_user_view.person.id;

  check_local_vote_mode(
    data.is_upvote,
    PostOrCommentId::Comment(comment_id),
    &local_site,
    my_person_id,
    &mut context.pool(),
  )
  .await?;
  check_bot_account(&local_user_view.person)?;

  let orig_comment = CommentView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;
  let previous_is_upvote = orig_comment.comment_actions.and_then(|p| p.vote_is_upvote);

  check_community_user_action(
    &local_user_view,
    &orig_comment.community,
    &mut context.pool(),
  )
  .await?;

  // Remove any likes first
  CommentActions::remove_like(&mut context.pool(), my_person_id, comment_id).await?;
  if let Some(previous_is_upvote) = previous_is_upvote {
    PersonActions::remove_like(
      &mut context.pool(),
      my_person_id,
      orig_comment.creator.id,
      previous_is_upvote,
    )
    .await
    // Ignore errors, since a previous_like of zero throws an error
    .ok();
  }

  if let Some(is_upvote) = data.is_upvote {
    let mut like_form = CommentLikeForm::new(my_person_id, data.comment_id, is_upvote);
    like_form = plugin_hook_before("before_comment_vote", like_form).await?;
    let like = CommentActions::like(&mut context.pool(), &like_form).await?;
    PersonActions::like(
      &mut context.pool(),
      my_person_id,
      orig_comment.creator.id,
      like_form.vote_is_upvote,
    )
    .await?;

    plugin_hook_after("after_comment_vote", &like)?;
  }

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
