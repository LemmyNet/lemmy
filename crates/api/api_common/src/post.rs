pub use lemmy_db_schema::{
  newtypes::PostId,
  source::post::{Post, PostActions},
  PostFeatureType,
};
pub use lemmy_db_schema_file::enums::PostListingMode;
pub use lemmy_db_views_post::{
  api::{
    GetPost,
    GetPostResponse,
    GetPosts,
    GetPostsResponse,
    GetSiteMetadata,
    GetSiteMetadataResponse,
    LinkMetadata,
    OpenGraphData,
    PostResponse,
  },
  PostView,
};

pub mod actions {
  pub use lemmy_db_views_post::api::{
    CreatePost,
    CreatePostLike,
    DeletePost,
    EditPost,
    HidePost,
    MarkManyPostsAsRead,
    MarkPostAsRead,
    SavePost,
  };

  pub mod moderation {
    pub use lemmy_db_views_post::api::{
      FeaturePost,
      ListPostLikes,
      ListPostLikesResponse,
      LockPost,
      PurgePost,
      RemovePost,
    };
  }
}
