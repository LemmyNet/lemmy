use lemmy_db_schema::newtypes::{CommentId, PostId};
use lemmy_db_views_community::CommunityView;
use lemmy_db_views_post::PostView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
// TODO this should be made into a tagged enum
/// Get a post. Needs either the post id, or comment_id.
pub struct GetPost {
  pub id: Option<PostId>,
  pub comment_id: Option<CommentId>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The post response.
pub struct GetPostResponse {
  pub post_view: PostView,
  pub community_view: CommunityView,
  /// A list of cross-posts, or other times / communities this link has been posted to.
  pub cross_posts: Vec<PostView>,
}
