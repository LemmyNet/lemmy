use crate::newtypes::SiteRoleId;
#[cfg(feature = "full")]
use crate::schema::site_role;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Identifiable, Insertable, AsChangeset, TS)
)]
#[cfg_attr(feature = "full", diesel(table_name = site_role))]
#[cfg_attr(feature = "full", ts(export))]
/// A site role
pub struct SiteRole {
  pub id: SiteRoleId,
  pub name: String,
  pub configure_site_roles: bool,
  pub assign_user_roles: bool,
  pub update_site_details: bool,
  pub hide_community: bool,
  pub transfer_community: bool,
  pub feature_post: bool,
  pub create_community: bool,
  pub remove_community: bool,
  pub modify_community: bool,
  pub view_removed_content: bool,
  pub distinguish_comment: bool,
  pub remove_comment: bool,
  pub remove_post: bool,
  pub lock_unlock_post: bool,
  pub manage_community_mods: bool,
  pub ban_person: bool,
  pub view_banned_persons: bool,
  pub view_private_message_reports: bool,
  pub resolve_private_message_reports: bool,
  pub view_post_reports: bool,
  pub resolve_post_reports: bool,
  pub view_comment_reports: bool,
  pub resolve_comment_reports: bool,
  pub approve_registration: bool,
  pub view_registration: bool,
  pub purge_comment: bool,
  pub purge_community: bool,
  pub purge_person: bool,
  pub purge_post: bool,
  pub view_modlog_names: bool,
  pub modify_custom_emoji: bool,
  pub unblockable: bool,
}
