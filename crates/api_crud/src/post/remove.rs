use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  post::{PostResponse, RemovePost},
  utils::{blocking, check_community_ban, get_local_user_view_from_jwt, is_mod_or_admin},
};
use lemmy_apub::activities::deletion::{send_apub_delete_in_community, DeletableObjects};
use lemmy_db_schema::{
  source::{
    community::Community,
    moderator::{ModRemovePost, ModRemovePostForm},
    post::Post,
  },
  traits::Crud,
};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::{send::send_post_ws_message, LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl PerformCrud for RemovePost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &RemovePost = self;
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

    // Verify that only the mods can remove
    is_mod_or_admin(
      context.pool(),
      local_user_view.person.id,
      orig_post.community_id,
    )
    .await?;

    // Update the post
    let post_id = data.post_id;
    let removed = data.removed;
    let updated_post = blocking(context.pool(), move |conn| {
      Post::update_removed(conn, post_id, removed)
    })
    .await??;

    // Mod tables
    let form = ModRemovePostForm {
      mod_person_id: local_user_view.person.id,
      post_id: data.post_id,
      removed: Some(removed),
      reason: data.reason.to_owned(),
    };
    blocking(context.pool(), move |conn| {
      ModRemovePost::create(conn, &form)
    })
    .await??;

    let res = send_post_ws_message(
      data.post_id,
      UserOperationCrud::RemovePost,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await?;

    // apub updates
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, orig_post.community_id)
    })
    .await??;
    let deletable = DeletableObjects::Post(Box::new(updated_post.into()));
    send_apub_delete_in_community(
      local_user_view.person,
      community,
      deletable,
      data.reason.clone().or_else(|| Some("".to_string())),
      removed,
      context,
    )
    .await?;
    Ok(res)
  }
}
