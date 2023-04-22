use crate::sensitive::Sensitive;
use lemmy_db_schema::{
  newtypes::{CommunityId, DbUrl, LanguageId, PostId, PostOrCommentId, PostReportId},
  ListingType,
  PostFeatureType,
  SortType,
};
use lemmy_db_views::structs::{PostReportView, PostView};
use lemmy_db_views_actor::structs::{CommunityModeratorView, CommunityView};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use url::Url;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct CreatePost {
  pub name: String,
  pub community_id: CommunityId,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub url: Option<Url>,
  pub body: Option<String>,
  pub honeypot: Option<String>,
  pub nsfw: Option<bool>,
  pub language_id: Option<LanguageId>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PostResponse {
  pub post_view: PostView,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetPost {
  pub id: PostOrCommentId,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Option<Sensitive<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetPostResponse {
  pub post_view: PostView,
  pub community_view: CommunityView,
  pub moderators: Vec<CommunityModeratorView>,
  pub cross_posts: Vec<PostView>,
  pub online: usize,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetPosts {
  pub type_: Option<ListingType>,
  pub sort: Option<SortType>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub community_id: Option<CommunityId>,
  pub community_name: Option<String>,
  pub saved_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Option<Sensitive<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetPostsResponse {
  pub posts: Vec<PostView>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct CreatePostLike {
  pub post_id: PostId,
  pub score: i16,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct EditPost {
  pub post_id: PostId,
  pub name: Option<String>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub url: Option<Url>,
  pub body: Option<String>,
  pub nsfw: Option<bool>,
  pub language_id: Option<LanguageId>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct DeletePost {
  pub post_id: PostId,
  pub deleted: bool,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct RemovePost {
  pub post_id: PostId,
  pub removed: bool,
  pub reason: Option<String>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct MarkPostAsRead {
  pub post_id: PostId,
  pub read: bool,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct LockPost {
  pub post_id: PostId,
  pub locked: bool,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct FeaturePost {
  pub post_id: PostId,
  pub featured: bool,
  pub feature_type: PostFeatureType,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct SavePost {
  pub post_id: PostId,
  pub save: bool,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct CreatePostReport {
  pub post_id: PostId,
  pub reason: String,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PostReportResponse {
  pub post_report_view: PostReportView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ResolvePostReport {
  pub report_id: PostReportId,
  pub resolved: bool,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ListPostReports {
  pub page: Option<i64>,
  pub limit: Option<i64>,
  /// Only shows the unresolved reports
  pub unresolved_only: Option<bool>,
  /// if no community is given, it returns reports for all communities moderated by the auth user
  pub community_id: Option<CommunityId>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub auth: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ListPostReportsResponse {
  pub post_reports: Vec<PostReportView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetSiteMetadata {
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub url: Url,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct GetSiteMetadataResponse {
  pub metadata: SiteMetadata,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct SiteMetadata {
  pub title: Option<String>,
  pub description: Option<String>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub(crate) image: Option<DbUrl>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  pub embed_video_url: Option<DbUrl>,
}
