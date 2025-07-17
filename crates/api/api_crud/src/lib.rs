use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::community::{Community, CommunityActions};

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
  CommunityActions::check_accept_activity_in_community(&mut context.pool(), community.id)
    .await
    .is_ok()
}
