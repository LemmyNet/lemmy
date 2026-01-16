use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::{
  community::{Community, CommunityActions},
  person::Person,
};
use lemmy_utils::error::LemmyResult;

pub mod comment;
pub mod community;
pub mod custom_emoji;
pub mod multi_community;
pub mod oauth_provider;
pub mod post;
pub mod private_message;
pub mod site;
pub mod tagline;
pub mod user;

/// Only mark new posts/comments to remote community as pending if it has any local followers.
/// Otherwise it could never get updated to be marked as published.
async fn community_use_pending(community: &Community, context: &LemmyContext) -> bool {
  if community.local {
    return false;
  }
  CommunityActions::check_accept_activity_in_community(&mut context.pool(), community)
    .await
    .is_ok()
}

async fn check_user_or_community_name_taken(name: &str, context: &LemmyContext) -> LemmyResult<()> {
  // TODO: better to make only a single sql query
  Person::check_name_taken(&mut context.pool(), &name).await?;
  Community::check_name_taken(&mut context.pool(), &name).await?;
  Ok(())
}
