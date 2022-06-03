use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  post::{LockPost, PostResponse},
  utils::{
    blocking,
    check_community_ban,
    check_community_deleted_or_removed,
    get_local_user_view_from_jwt,
    is_mod_or_admin,
  },
};
use lemmy_apub::{
  objects::post::ApubPost,
  protocol::activities::{create_or_update::post::CreateOrUpdatePost, CreateOrUpdateType},
};
use lemmy_db_schema::{
  source::{
    moderator::{ModLockPost, ModLockPostForm},
    post::Post,
  },
  traits::Crud,
};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::{send::send_post_ws_message, LemmyContext, UserOperation};

#[async_trait::async_trait(?Send)]
impl Perform for LockPost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &LockPost = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let post_id = data.post_id;
    let orig_post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    check_community_ban(
      local_user_view.person.id,
      orig_post.community_id,
      context.pool(),
    )
    .await?;
    check_community_deleted_or_removed(orig_post.community_id, context.pool()).await?;

    // Verify that only the mods can lock
    is_mod_or_admin(
      context.pool(),
      local_user_view.person.id,
      orig_post.community_id,
    )
    .await?;

    // Update the post
    let post_id = data.post_id;
    let locked = data.locked;
    let updated_post: ApubPost = blocking(context.pool(), move |conn| {
      Post::update_locked(conn, post_id, locked)
    })
    .await??
    .into();

    // Mod tables
    let form = ModLockPostForm {
      mod_person_id: local_user_view.person.id,
      post_id: data.post_id,
      locked: Some(locked),
    };
    blocking(context.pool(), move |conn| ModLockPost::create(conn, &form)).await??;

    // apub updates
    CreateOrUpdatePost::send(
      updated_post,
      &local_user_view.person.clone().into(),
      CreateOrUpdateType::Update,
      context,
    )
    .await?;

    send_post_ws_message(
      data.post_id,
      UserOperation::LockPost,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await
  }
}
