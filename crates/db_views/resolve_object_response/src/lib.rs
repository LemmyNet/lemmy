use lemmy_db_views_comment::CommentView;
use lemmy_db_views_community::CommunityView;
use lemmy_db_views_person::PersonView;
use lemmy_db_views_post::PostView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
// TODO Change this to an enum
/// The response of an apub object fetch.
pub struct ResolveObjectResponse {
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment: Option<CommentView>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub post: Option<PostView>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub community: Option<CommunityView>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub person: Option<PersonView>,
}
