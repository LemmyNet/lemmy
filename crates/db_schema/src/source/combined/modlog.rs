use crate::newtypes::{
  AdminAllowInstanceId,
  AdminBlockInstanceId,
  AdminPurgeCommentId,
  AdminPurgeCommunityId,
  AdminPurgePersonId,
  AdminPurgePostId,
  ModAddCommunityId,
  ModAddId,
  ModBanFromCommunityId,
  ModBanId,
  ModFeaturePostId,
  ModHideCommunityId,
  ModLockPostId,
  ModRemoveCommentId,
  ModRemoveCommunityId,
  ModRemovePostId,
  ModTransferCommunityId,
  ModlogCombinedId,
};
#[cfg(feature = "full")]
use crate::schema::modlog_combined;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = modlog_combined))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = modlog_combined_keys))]
/// A combined modlog table.
pub struct ModlogCombined {
  pub id: ModlogCombinedId,
  pub published: DateTime<Utc>,
  pub admin_allow_instance_id: Option<AdminAllowInstanceId>,
  pub admin_block_instance_id: Option<AdminBlockInstanceId>,
  pub admin_purge_comment_id: Option<AdminPurgeCommentId>,
  pub admin_purge_community_id: Option<AdminPurgeCommunityId>,
  pub admin_purge_person_id: Option<AdminPurgePersonId>,
  pub admin_purge_post_id: Option<AdminPurgePostId>,
  pub mod_add_id: Option<ModAddId>,
  pub mod_add_community_id: Option<ModAddCommunityId>,
  pub mod_ban_id: Option<ModBanId>,
  pub mod_ban_from_community_id: Option<ModBanFromCommunityId>,
  pub mod_feature_post_id: Option<ModFeaturePostId>,
  pub mod_hide_community_id: Option<ModHideCommunityId>,
  pub mod_lock_post_id: Option<ModLockPostId>,
  pub mod_remove_comment_id: Option<ModRemoveCommentId>,
  pub mod_remove_community_id: Option<ModRemoveCommunityId>,
  pub mod_remove_post_id: Option<ModRemovePostId>,
  pub mod_transfer_community_id: Option<ModTransferCommunityId>,
}
