use crate::{context::LemmyContext, utils::check_community_mod_action};
use lemmy_db_schema::{
  newtypes::TagId,
  source::{post::Post, post_tag::PostTag, tag::PostTagInsertForm},
};
use lemmy_db_views::structs::{CommunityView, LocalUserView};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use std::collections::HashSet;

pub async fn update_post_tags(
  context: &LemmyContext,
  post: &Post,
  community: &CommunityView,
  tags: &[TagId],
  local_user_view: &LocalUserView,
) -> LemmyResult<()> {
  let is_author = Post::is_post_creator(local_user_view.person.id, post.creator_id);

  if !is_author {
    // Check if user is either the post author or a community mod
    check_community_mod_action(
      local_user_view,
      &community.community,
      false,
      &mut context.pool(),
    )
    .await?;
  }
  // validate tags
  let valid_tags: HashSet<TagId> = community.post_tags.0.iter().map(|t| t.id).collect();
  if !valid_tags.is_superset(&tags.iter().copied().collect()) {
    return Err(LemmyErrorType::TagNotInCommunity.into());
  }
  let insert_tags = tags
    .iter()
    .map(|tag_id| PostTagInsertForm {
      post_id: post.id,
      tag_id: *tag_id,
    })
    .collect();
  PostTag::set(&mut context.pool(), post.id, insert_tags).await?;
  Ok(())
}
