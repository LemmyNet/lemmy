pub use lemmy_db_schema::{
  newtypes::{CommunityId, MultiCommunityId, TagId},
  source::{
    community::{Community, CommunityActions},
    multi_community::{MultiCommunity, MultiCommunityFollow},
    tag::{Tag, TagsView},
  },
};
pub use lemmy_db_schema_file::enums::CommunityVisibility;
pub use lemmy_db_views_community::{
  api::{
    CommunityResponse,
    CreateMultiCommunity,
    CreateOrDeleteMultiCommunityEntry,
    FollowMultiCommunity,
    GetCommunity,
    GetCommunityResponse,
    GetMultiCommunity,
    GetMultiCommunityResponse,
    GetRandomCommunity,
    ListCommunities,
    ListCommunitiesResponse,
    ListMultiCommunities,
    ListMultiCommunitiesResponse,
    UpdateCommunityNotifications,
    UpdateMultiCommunity,
  },
  CommunityView,
  MultiCommunityView,
};
pub use lemmy_db_views_community_follower_approval::PendingFollowerView;
pub use lemmy_db_views_community_moderator::CommunityModeratorView;

pub mod actions {
  pub use lemmy_db_views_community::api::{
    BlockCommunity,
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
      UpdateCommunityTag,
    };
    pub use lemmy_db_views_community_follower::CommunityFollowerView;
    pub use lemmy_db_views_community_follower_approval::{
      api::{
        GetCommunityPendingFollowsCountResponse,
        ListCommunityPendingFollows,
        ListCommunityPendingFollowsResponse,
      },
      PendingFollowerView,
    };
  }
}
