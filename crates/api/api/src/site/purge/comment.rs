use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::is_admin,
};
use lemmy_db_schema::source::{
  comment::Comment,
  local_user::LocalUser,
  modlog::{Modlog, ModlogInsertForm},
};
use lemmy_db_views_comment::{CommentView, api::PurgeComment};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::LemmyResult;

pub async fn purge_comment(
  Json(data): Json<PurgeComment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  // Only let admin purge an item
  is_admin(&local_user_view)?;

  let comment_id = data.comment_id;
  let local_instance_id = local_user_view.person.instance_id;

  // Read the comment to get the post_id and community
  let comment_view = CommentView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  // Also check that you're a higher admin
  LocalUser::is_higher_admin_check(
    &mut context.pool(),
    local_user_view.person.id,
    vec![comment_view.creator.id],
  )
  .await?;

  // TODO read comments for pictrs images and purge them

  Comment::delete(&mut context.pool(), comment_id).await?;

  // Mod tables
  let form = ModlogInsertForm::admin_purge_comment(
    local_user_view.person.id,
    &comment_view.comment,
    comment_view.community.id,
    &data.reason,
  );
  Modlog::create(&mut context.pool(), &[form]).await?;

  ActivityChannel::submit_activity(
    SendActivityData::RemoveComment {
      comment: comment_view.comment,
      moderator: local_user_view.person.clone(),
      community: comment_view.community,
      reason: data.reason.clone(),
      with_replies: false,
    },
    &context,
  )?;

  Ok(Json(SuccessResponse::default()))
}
