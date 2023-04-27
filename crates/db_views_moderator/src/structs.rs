use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  source::{
    comment::Comment,
    community::Community,
    moderator::{
      AdminPurgeComment,
      AdminPurgeCommunity,
      AdminPurgePerson,
      AdminPurgePost,
      ModAdd,
      ModAddCommunity,
      ModBan,
      ModBanFromCommunity,
      ModFeaturePost,
      ModHideCommunity,
      ModLockPost,
      ModRemoveComment,
      ModRemoveCommunity,
      ModRemovePost,
      ModTransferCommunity,
    },
    person::Person,
    post::Post,
  },
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ModAddCommunityView {
  pub mod_add_community: ModAddCommunity,
  pub moderator: Option<Person>,
  pub community: Community,
  pub modded_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ModAddView {
  pub mod_add: ModAdd,
  pub moderator: Option<Person>,
  pub modded_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ModBanFromCommunityView {
  pub mod_ban_from_community: ModBanFromCommunity,
  pub moderator: Option<Person>,
  pub community: Community,
  pub banned_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ModBanView {
  pub mod_ban: ModBan,
  pub moderator: Option<Person>,
  pub banned_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ModHideCommunityView {
  pub mod_hide_community: ModHideCommunity,
  pub admin: Option<Person>,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ModLockPostView {
  pub mod_lock_post: ModLockPost,
  pub moderator: Option<Person>,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ModRemoveCommentView {
  pub mod_remove_comment: ModRemoveComment,
  pub moderator: Option<Person>,
  pub comment: Comment,
  pub commenter: Person,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ModRemoveCommunityView {
  pub mod_remove_community: ModRemoveCommunity,
  pub moderator: Option<Person>,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ModRemovePostView {
  pub mod_remove_post: ModRemovePost,
  pub moderator: Option<Person>,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ModFeaturePostView {
  pub mod_feature_post: ModFeaturePost,
  pub moderator: Option<Person>,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ModTransferCommunityView {
  pub mod_transfer_community: ModTransferCommunity,
  pub moderator: Option<Person>,
  pub community: Community,
  pub modded_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AdminPurgeCommentView {
  pub admin_purge_comment: AdminPurgeComment,
  pub admin: Option<Person>,
  pub post: Post,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AdminPurgeCommunityView {
  pub admin_purge_community: AdminPurgeCommunity,
  pub admin: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AdminPurgePersonView {
  pub admin_purge_person: AdminPurgePerson,
  pub admin: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AdminPurgePostView {
  pub admin_purge_post: AdminPurgePost,
  pub admin: Option<Person>,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ModlogListParams {
  pub community_id: Option<CommunityId>,
  pub mod_person_id: Option<PersonId>,
  pub other_person_id: Option<PersonId>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub hide_modlog_names: bool,
}
