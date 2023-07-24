use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{FeaturePost, PostResponse},
  utils::{
    check_community_ban,
    check_community_deleted_or_removed,
    is_admin,
    is_mod_or_admin,
    local_user_view_from_jwt,
  },
};
use lemmy_db_schema::{
  source::{
    moderator::{ModFeaturePost, ModFeaturePostForm},
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
  PostFeatureType,
};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for FeaturePost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<PostResponse, LemmyError> {
    let data: &FeaturePost = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let post_id = data.post_id;
    let orig_post = Post::read(&mut context.pool(), post_id).await?;

    check_community_ban(
      local_user_view.person.id,
      orig_post.community_id,
      &mut context.pool(),
    )
    .await?;
    check_community_deleted_or_removed(orig_post.community_id, &mut context.pool()).await?;

    if data.feature_type == PostFeatureType::Community {
      // Verify that only the mods can feature in community
      is_mod_or_admin(
        &mut context.pool(),
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
    Post::update(&mut context.pool(), post_id, &new_post).await?;

    // Mod tables
    let form = ModFeaturePostForm {
      mod_person_id: local_user_view.person.id,
      post_id: data.post_id,
      featured: data.featured,
      is_featured_community: data.feature_type == PostFeatureType::Community,
    };

    ModFeaturePost::create(&mut context.pool(), form).await?;

    build_post_response(
      context,
      orig_post.community_id,
      local_user_view.person.id,
      post_id,
    )
    .await
  }
}
