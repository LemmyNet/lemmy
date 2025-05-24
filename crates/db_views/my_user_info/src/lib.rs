use lemmy_db_schema::{
  newtypes::LanguageId,
  source::{community::Community, instance::Instance, person::Person},
};
use lemmy_db_views_community_follower::CommunityFollowerView;
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Your user info.
pub struct MyUserInfo {
  pub local_user_view: LocalUserView,
  pub follows: Vec<CommunityFollowerView>,
  pub moderates: Vec<CommunityModeratorView>,
  pub community_blocks: Vec<Community>,
  pub instance_blocks: Vec<Instance>,
  pub person_blocks: Vec<Person>,
  pub keyword_blocks: Vec<String>,
  pub discussion_languages: Vec<LanguageId>,
}
