use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  build_response::build_comment_response,
  context::LemmyContext,
  plugins::{plugin_hook_after, plugin_hook_before},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_bot_account, check_community_user_action, check_local_vote_mode},
};
use lemmy_db_schema::{
  newtypes::PostOrCommentId,
  source::comment::{CommentActions, CommentLikeForm},
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
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  let local_instance_id = local_user_view.person.instance_id;
  let comment_id = data.comment_id;

  check_local_vote_mode(
    data.score,
    PostOrCommentId::Comment(comment_id),
    &local_site,
    local_user_view.person.id,
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

  check_community_user_action(
    &local_user_view,
    &orig_comment.community,
    &mut context.pool(),
  )
  .await?;

  let mut like_form = CommentLikeForm::new(local_user_view.person.id, data.comment_id, data.score);

  // Remove any likes first
  let person_id = local_user_view.person.id;

  CommentActions::remove_like(&mut context.pool(), person_id, comment_id).await?;

  // Only add the like if the score isnt 0
  let do_add =
    like_form.like_score != 0 && (like_form.like_score == 1 || like_form.like_score == -1);
  if do_add {
    like_form = plugin_hook_before("before_comment_vote", like_form).await?;
    let like = CommentActions::like(&mut context.pool(), &like_form).await?;
    plugin_hook_after("after_comment_vote", &like)?;
  }

  ActivityChannel::submit_activity(
    SendActivityData::LikePostOrComment {
      object_id: orig_comment.comment.ap_id,
      actor: local_user_view.person.clone(),
      community: orig_comment.community,
      score: data.score,
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
