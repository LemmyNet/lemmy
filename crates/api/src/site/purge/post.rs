use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  request::purge_image_from_pictrs,
  site::{PurgeItemResponse, PurgePost},
  utils::{is_admin, local_user_view_from_jwt, sanitize_html_api_opt},
};
use lemmy_db_schema::{
  source::{
    moderator::{AdminPurgePost, AdminPurgePostForm},
    post::Post,
  },
  traits::Crud,
};
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn purge_post(
  data: Json<PurgePost>,
  context: Data<LemmyContext>,
) -> Result<Json<PurgeItemResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

  // Only let admin purge an item
  is_admin(&local_user_view)?;

  let post_id = data.post_id;

  // Read the post to get the community_id
  let post = Post::read(&mut context.pool(), post_id).await?;

  // Purge image
  if let Some(url) = post.url {
    purge_image_from_pictrs(&url, &context).await.ok();
  }
  // Purge thumbnail
  if let Some(thumbnail_url) = post.thumbnail_url {
    purge_image_from_pictrs(&thumbnail_url, &context).await.ok();
  }

  let community_id = post.community_id;

  Post::delete(&mut context.pool(), post_id).await?;

  // Mod tables
  let reason = sanitize_html_api_opt(&data.reason);
  let form = AdminPurgePostForm {
    admin_person_id: local_user_view.person.id,
    reason,
    community_id,
  };

  AdminPurgePost::create(&mut context.pool(), &form).await?;

  Ok(Json(PurgeItemResponse { success: true }))
}
