use lemmy_db_schema::source::community::Community;
use lemmy_db_schema_file::enums::{CommunityFollowerState, CommunityVisibility};

pub mod add_mod;
pub mod ban;
pub mod block;
pub mod follow;
pub mod multi_community_follow;
pub mod pending_follows;
pub mod random;
pub mod tag;
pub mod transfer;

fn community_follower_state(community: &Community) -> CommunityFollowerState {
  if community.local {
    // Local follow is accepted immediately
    CommunityFollowerState::Accepted
  } else if community.visibility == CommunityVisibility::Private {
    // Private communities require manual approval
    CommunityFollowerState::ApprovalRequired
  } else {
    // remote follow needs to be federated first
    CommunityFollowerState::Pending
  }
}
