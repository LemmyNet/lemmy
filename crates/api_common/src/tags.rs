use crate::{context::LemmyContext, utils::check_community_mod_action};
use activitypub_federation::config::Data;
use lemmy_db_schema::{
  newtypes::TagId,
  source::{post::Post, post_tag::PostTag, tag::PostTagInsertForm},
};
use lemmy_db_views::structs::{CommunityView, LocalUserView};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn update_post_tags(
  context: &Data<LemmyContext>,
  post: &Post,
  community: &CommunityView,
  tags: &[TagId],
  local_user_view: &LocalUserView,
) -> LemmyResult<()> {
  let is_author = Post::is_post_creator(local_user_view.person.id, post.creator_id);

  if !is_author {
    // Check if user is either the post author or a community mod
    check_community_mod_action(
      &local_user_view.person,
      &community.community,
      false,
      &mut context.pool(),
    )
    .await?;
  }
  // validate tags
  let valid_tags: std::collections::HashSet<TagId> =
    community.post_tags.0.iter().map(|t| t.id).collect();
  if tags.iter().any(|tag_id| !valid_tags.contains(tag_id)) {
    return Err(LemmyErrorType::InvalidBodyField.into());
  }
  // Delete existing post tags
  PostTag::delete_for_post(&mut context.pool(), post.id).await?;
  // Create new post tags
  PostTag::create_many(
    &mut context.pool(),
    tags
      .iter()
      .map(|tag_id| PostTagInsertForm {
        post_id: post.id,
        tag_id: *tag_id,
      })
      .collect(),
  )
  .await?;
  Ok(())
}
