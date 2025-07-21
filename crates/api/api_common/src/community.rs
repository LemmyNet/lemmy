pub use lemmy_db_schema::{
  newtypes::{CommunityId, TagId},
  source::{
    community::{Community, CommunityActions},
    tag::{Tag, TagsView},
  },
};
pub use lemmy_db_schema_file::enums::CommunityVisibility;
pub use lemmy_db_views_community::{
  api::{
    CommunityResponse,
    GetCommunity,
    GetCommunityResponse,
    GetRandomCommunity,
    ListCommunities,
    ListCommunitiesResponse,
  },
  CommunityView,
};
pub use lemmy_db_views_community_follower::PendingFollow;
pub use lemmy_db_views_community_moderator::CommunityModeratorView;

pub mod actions {
  pub use lemmy_db_views_community::api::{
    BlockCommunity,
    BlockCommunityResponse,
    CreateCommunity,
    FollowCommunity,
    HideCommunity,
  };

  pub mod moderation {
    pub use lemmy_db_schema_file::enums::CommunityFollowerState;
    pub use lemmy_db_views_community::api::{
      AddModToCommunity,
      AddModToCommunityResponse,
      ApproveCommunityPendingFollower,
      BanFromCommunity,
      BanFromCommunityResponse,
      CommunityIdQuery,
      CreateCommunityTag,
      DeleteCommunity,
      DeleteCommunityTag,
      EditCommunity,
      PurgeCommunity,
      RemoveCommunity,
      TransferCommunity,
    };
    pub use lemmy_db_views_community_follower::{
      api::{
        GetCommunityPendingFollowsCount,
        GetCommunityPendingFollowsCountResponse,
        ListCommunityPendingFollows,
        ListCommunityPendingFollowsResponse,
      },
      CommunityFollowerView,
    };
  }
}
