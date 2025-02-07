use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{PostResponse, UpdatePostTags},
  utils::check_community_mod_action,
};
use lemmy_db_schema::{
  source::{community::Community, post::Post, post_tag::PostTag, tag::PostTagInsertForm},
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn update_post_tags(
  data: Json<UpdatePostTags>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let post = Post::read(&mut context.pool(), data.post_id).await?;
  
  let is_author = local_user_view.person.id == post.creator_id;
  
  if !is_author {
    let community = Community::read(&mut context.pool(), post.community_id).await?;
    // Check if user is either the post author or a community mod
    check_community_mod_action(
      &local_user_view.person,
      &community,
      false,
      &mut context.pool(),
    )
    .await?;
  }

  // Delete existing post tags
  PostTag::delete_for_post(&mut context.pool(), data.post_id).await?;

  // Create new post tags
  for tag_id in &data.tags {
    let form = PostTagInsertForm {
      post_id: data.post_id,
      tag_id: *tag_id,
    };
    PostTag::create(&mut context.pool(), &form).await?;
  }

  build_post_response(&context, post.community_id, local_user_view, data.post_id).await
}
