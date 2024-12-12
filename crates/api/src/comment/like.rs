use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_comment_response,
  comment::{CommentResponse, CreateCommentLike},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_bot_account, check_community_user_action, check_local_vote_mode},
};
use lemmy_db_schema::{
  newtypes::{LocalUserId, PostOrCommentId},
  source::{
    comment::{CommentLike, CommentLikeForm},
    comment_reply::CommentReply,
    local_site::LocalSite,
  },
  traits::Likeable,
};
use lemmy_db_views::structs::{CommentView, LocalUserView};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use std::ops::Deref;

#[tracing::instrument(skip(context))]
pub async fn like_comment(
  data: Json<CreateCommentLike>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponse>> {
  let local_site = LocalSite::read(&mut context.pool()).await?;
  let comment_id = data.comment_id;

  let mut recipient_ids = Vec::<LocalUserId>::new();

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
  )
  .await?;

  check_community_user_action(
    &local_user_view.person,
    &orig_comment.community,
    &mut context.pool(),
  )
  .await?;

  // Add parent poster or commenter to recipients
  let comment_reply = CommentReply::read_by_comment(&mut context.pool(), comment_id).await;
  if let Ok(Some(reply)) = comment_reply {
    let recipient_id = reply.recipient_id;
    if let Ok(local_recipient) = LocalUserView::read_person(&mut context.pool(), recipient_id).await
    {
      recipient_ids.push(local_recipient.local_user.id);
    }
  }

  let like_form = CommentLikeForm {
    comment_id: data.comment_id,
    person_id: local_user_view.person.id,
    score: data.score,
  };

  // Remove any likes first
  let person_id = local_user_view.person.id;

  CommentLike::remove(&mut context.pool(), person_id, comment_id).await?;

  // Only add the like if the score isnt 0
  let do_add = like_form.score != 0 && (like_form.score == 1 || like_form.score == -1);
  if do_add {
    CommentLike::like(&mut context.pool(), &like_form)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntLikeComment)?;
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
      recipient_ids,
    )
    .await?,
  ))
}
