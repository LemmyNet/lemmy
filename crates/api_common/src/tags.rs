use crate::{context::LemmyContext, utils::check_community_mod_action};
use activitypub_federation::config::Data;
use lemmy_db_schema::{
  newtypes::TagId,
  source::{community::Community, post::Post, post_tag::PostTag, tag::PostTagInsertForm},
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn update_post_tags(
  context: &Data<LemmyContext>,
  post: &Post,
  community: &Community,
  tags: &[TagId],
  local_user_view: &LocalUserView,
) -> LemmyResult<()> {
  let post = Post::read(&mut context.pool(), post.id).await?;

  let is_author = Post::is_post_creator(local_user_view.person.id, post.creator_id);

  if !is_author {
    // Check if user is either the post author or a community mod
    check_community_mod_action(
      &local_user_view.person,
      community,
      false,
      &mut context.pool(),
    )
    .await?;
  }

  // Delete existing post tags
  PostTag::delete_for_post(&mut context.pool(), post.id).await?;

  // Create new post tags
  for tag_id in tags {
    let form = PostTagInsertForm {
      post_id: post.id,
      tag_id: *tag_id,
    };
    PostTag::create(&mut context.pool(), &form).await?;
  }
  Ok(())
}
