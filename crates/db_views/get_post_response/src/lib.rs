use lemmy_db_views_community::CommunityView;
use lemmy_db_views_post::PostView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The post response.
pub struct GetPostResponse {
  pub post_view: PostView,
  pub community_view: CommunityView,
  /// A list of cross-posts, or other times / communities this link has been posted to.
  pub cross_posts: Vec<PostView>,
}
