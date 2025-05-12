use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  site::PurgePost,
  utils::{is_admin, purge_post_images},
  SuccessResponse,
};
use lemmy_db_schema::{
  source::{
    local_user::LocalUser,
    mod_log::admin::{AdminPurgePost, AdminPurgePostForm},
    post::Post,
  },
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn purge_post(
  data: Json<PurgePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  // Only let admin purge an item
  is_admin(&local_user_view)?;

  // Read the post to get the community_id
  let post = Post::read(&mut context.pool(), data.post_id).await?;

  // Also check that you're a higher admin
  LocalUser::is_higher_admin_check(
    &mut context.pool(),
    local_user_view.person.id,
    vec![post.creator_id],
  )
  .await?;

  purge_post_images(post.url.clone(), post.thumbnail_url.clone(), &context).await;

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
  )?;

  Ok(Json(SuccessResponse::default()))
}
