use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  post::{PostResponse, StickyPost},
  utils::{
    check_community_ban,
    check_community_deleted_or_removed,
    get_local_user_view_from_jwt,
    is_mod_or_admin,
  },
  websocket::{send::send_post_ws_message, UserOperation},
};
use lemmy_db_schema::{
  source::{
    moderator::{ModStickyPost, ModStickyPostForm},
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for StickyPost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &StickyPost = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let post_id = data.post_id;
    let orig_post = Post::read(context.pool(), post_id).await?;

    check_community_ban(
      local_user_view.person.id,
      orig_post.community_id,
      context.pool(),
    )
    .await?;
    check_community_deleted_or_removed(orig_post.community_id, context.pool()).await?;

    // Verify that only the mods can sticky
    is_mod_or_admin(
      context.pool(),
      local_user_view.person.id,
      orig_post.community_id,
    )
    .await?;

    // Update the post
    let post_id = data.post_id;
    let stickied = data.stickied;
    Post::update(
      context.pool(),
      post_id,
      &PostUpdateForm::builder().stickied(Some(stickied)).build(),
    )
    .await?;

    // Mod tables
    let form = ModStickyPostForm {
      mod_person_id: local_user_view.person.id,
      post_id: data.post_id,
      stickied: Some(stickied),
    };

    ModStickyPost::create(context.pool(), &form).await?;

    send_post_ws_message(
      data.post_id,
      UserOperation::StickyPost,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await
  }
}
