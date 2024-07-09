use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  request::purge_image_from_pictrs,
  send_activity::{ActivityChannel, SendActivityData},
  site::PurgePost,
  utils::{check_is_higher_admin, is_admin},
  SuccessResponse,
};
use lemmy_db_schema::{
  source::{
    moderator::{AdminPurgePost, AdminPurgePostForm},
    post::Post,
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn purge_post(
  data: Json<PurgePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  // Only let admin purge an item
  is_admin(&local_user_view)?;

  // Read the post to get the community_id
  let post = Post::read(&mut context.pool(), data.post_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPost)?;

  // Also check that you're a higher admin
  check_is_higher_admin(&mut context.pool(), &local_user_view, vec![post.creator_id]).await?;

  // Purge image
  if let Some(url) = &post.url {
    purge_image_from_pictrs(url, &context).await.ok();
  }
  // Purge thumbnail
  if let Some(thumbnail_url) = &post.thumbnail_url {
    purge_image_from_pictrs(thumbnail_url, &context).await.ok();
  }

  Post::delete(&mut context.pool(), data.post_id).await?;

  // Mod tables
  let form = AdminPurgePostForm {
    admin_person_id: local_user_view.person.id,
    reason: data.reason.clone(),
    community_id: post.community_id,
  };
  AdminPurgePost::create(&mut context.pool(), &form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::RemovePost {
      post,
      moderator: local_user_view.person.clone(),
      reason: data.reason.clone(),
      removed: true,
    },
    &context,
  )
  .await?;

  Ok(Json(SuccessResponse::default()))
}
