use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  post::{FeaturePost, PostResponse},
  utils::{
    check_community_ban,
    check_community_deleted_or_removed,
    get_local_user_view_from_jwt,
    is_admin,
    is_mod_or_admin,
  },
  websocket::UserOperation,
};
use lemmy_db_schema::{
  source::{
    moderator::{ModFeaturePost, ModFeaturePostForm},
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
  PostFeatureType,
};
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for FeaturePost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &FeaturePost = self;
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

    if data.feature_type == PostFeatureType::Community {
      // Verify that only the mods can feature in community
      is_mod_or_admin(
        context.pool(),
        local_user_view.person.id,
        orig_post.community_id,
      )
      .await?;
    } else {
      is_admin(&local_user_view)?;
    }

    // Update the post
    let post_id = data.post_id;
    let new_post: PostUpdateForm = if data.feature_type == PostFeatureType::Community {
      PostUpdateForm::builder()
        .featured_community(Some(data.featured))
        .build()
    } else {
      PostUpdateForm::builder()
        .featured_local(Some(data.featured))
        .build()
    };
    Post::update(context.pool(), post_id, &new_post).await?;

    // Mod tables
    let form = ModFeaturePostForm {
      mod_person_id: local_user_view.person.id,
      post_id: data.post_id,
      featured: data.featured,
      is_featured_community: data.feature_type == PostFeatureType::Community,
    };

    ModFeaturePost::create(context.pool(), &form).await?;

    context
      .send_post_ws_message(
        &UserOperation::FeaturePost,
        data.post_id,
        websocket_id,
        Some(local_user_view.person.id),
      )
      .await
  }
}
