use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  request::purge_image_from_pictrs,
  site::{PurgeItemResponse, PurgePost},
  utils::{blocking, get_local_user_view_from_jwt, is_admin},
};
use lemmy_db_schema::{
  source::{
    moderator::{AdminPurgePost, AdminPurgePostForm},
    post::Post,
  },
  traits::Crud,
};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for PurgePost {
  type Response = PurgeItemResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data: &Self = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Only let admins purge an item
    is_admin(&local_user_view)?;

    let post_id = data.post_id;

    // Read the post to get the community_id
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    // Purge image
    if let Some(url) = post.url {
      purge_image_from_pictrs(context.client(), context.settings(), &url)
        .await
        .ok();
    }
    // Purge thumbnail
    if let Some(thumbnail_url) = post.thumbnail_url {
      purge_image_from_pictrs(context.client(), context.settings(), &thumbnail_url)
        .await
        .ok();
    }

    let community_id = post.community_id;

    blocking(context.pool(), move |conn| Post::delete(conn, post_id)).await??;

    // Mod tables
    let reason = data.reason.to_owned();
    let form = AdminPurgePostForm {
      admin_person_id: local_user_view.person.id,
      reason,
      community_id,
    };

    blocking(context.pool(), move |conn| {
      AdminPurgePost::create(conn, &form)
    })
    .await??;

    Ok(PurgeItemResponse { success: true })
  }
}
