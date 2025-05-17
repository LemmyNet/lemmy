#[cfg(feature = "full")]
use diesel::Queryable;
use lemmy_db_schema::source::{
  comment::Comment,
  community::Community,
  instance::Instance,
  mod_log::{
    admin::{
      AdminAllowInstance,
      AdminBlockInstance,
      AdminPurgeComment,
      AdminPurgeCommunity,
      AdminPurgePerson,
      AdminPurgePost,
    },
    moderator::{
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
  },
  person::Person,
  post::Post,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When someone is added as a community moderator.
pub struct ModAddCommunityView {
  pub mod_add_community: ModAddCommunity,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub community: Community,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When someone is added as a site moderator.
pub struct ModAddView {
  pub mod_add: ModAdd,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When someone is banned from a community.
pub struct ModBanFromCommunityView {
  pub mod_ban_from_community: ModBanFromCommunity,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub community: Community,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When someone is banned from the site.
pub struct ModBanView {
  pub mod_ban: ModBan,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a community is hidden from public view.
pub struct ModHideCommunityView {
  pub mod_hide_community: ModHideCommunity,
  #[cfg_attr(feature = "full", ts(optional))]
  pub admin: Option<Person>,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator locks a post (prevents new comments being made).
pub struct ModLockPostView {
  pub mod_lock_post: ModLockPost,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator removes a comment.
pub struct ModRemoveCommentView {
  pub mod_remove_comment: ModRemoveComment,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub comment: Comment,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator removes a community.
pub struct ModRemoveCommunityView {
  pub mod_remove_community: ModRemoveCommunity,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator removes a post.
pub struct ModRemovePostView {
  pub mod_remove_post: ModRemovePost,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator features a post on a community (pins it to the top).
pub struct ModFeaturePostView {
  pub mod_feature_post: ModFeaturePost,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator transfers a community to a new owner.
pub struct ModTransferCommunityView {
  pub mod_transfer_community: ModTransferCommunity,
  #[cfg_attr(feature = "full", ts(optional))]
  pub moderator: Option<Person>,
  pub community: Community,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a comment.
pub struct AdminPurgeCommentView {
  pub admin_purge_comment: AdminPurgeComment,
  #[cfg_attr(feature = "full", ts(optional))]
  pub admin: Option<Person>,
  pub post: Post,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a community.
pub struct AdminPurgeCommunityView {
  pub admin_purge_community: AdminPurgeCommunity,
  #[cfg_attr(feature = "full", ts(optional))]
  pub admin: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a person.
pub struct AdminPurgePersonView {
  pub admin_purge_person: AdminPurgePerson,
  #[cfg_attr(feature = "full", ts(optional))]
  pub admin: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a post.
pub struct AdminPurgePostView {
  pub admin_purge_post: AdminPurgePost,
  #[cfg_attr(feature = "full", ts(optional))]
  pub admin: Option<Person>,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a post.
pub struct AdminBlockInstanceView {
  pub admin_block_instance: AdminBlockInstance,
  pub instance: Instance,
  #[cfg_attr(feature = "full", ts(optional))]
  pub admin: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a post.
pub struct AdminAllowInstanceView {
  pub admin_allow_instance: AdminAllowInstance,
  pub instance: Instance,
  #[cfg_attr(feature = "full", ts(optional))]
  pub admin: Option<Person>,
}

/// like PaginationCursor but for the modlog_combined
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ModlogCombinedPaginationCursor(pub String);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined modlog view
pub struct ModlogCombinedViewInternal {
  // Specific
  pub admin_allow_instance: Option<AdminAllowInstance>,
  pub admin_block_instance: Option<AdminBlockInstance>,
  pub admin_purge_comment: Option<AdminPurgeComment>,
  pub admin_purge_community: Option<AdminPurgeCommunity>,
  pub admin_purge_person: Option<AdminPurgePerson>,
  pub admin_purge_post: Option<AdminPurgePost>,
  pub mod_add: Option<ModAdd>,
  pub mod_add_community: Option<ModAddCommunity>,
  pub mod_ban: Option<ModBan>,
  pub mod_ban_from_community: Option<ModBanFromCommunity>,
  pub mod_feature_post: Option<ModFeaturePost>,
  pub mod_hide_community: Option<ModHideCommunity>,
  pub mod_lock_post: Option<ModLockPost>,
  pub mod_remove_comment: Option<ModRemoveComment>,
  pub mod_remove_community: Option<ModRemoveCommunity>,
  pub mod_remove_post: Option<ModRemovePost>,
  pub mod_transfer_community: Option<ModTransferCommunity>,
  // Specific fields

  // Shared
  pub moderator: Option<Person>,
  pub other_person: Option<Person>,
  pub instance: Option<Instance>,
  pub community: Option<Community>,
  pub post: Option<Post>,
  pub comment: Option<Comment>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
// Use serde's internal tagging, to work easier with javascript libraries
#[serde(tag = "type_")]
pub enum ModlogCombinedView {
  AdminAllowInstance(AdminAllowInstanceView),
  AdminBlockInstance(AdminBlockInstanceView),
  AdminPurgeComment(AdminPurgeCommentView),
  AdminPurgeCommunity(AdminPurgeCommunityView),
  AdminPurgePerson(AdminPurgePersonView),
  AdminPurgePost(AdminPurgePostView),
  ModAdd(ModAddView),
  ModAddCommunity(ModAddCommunityView),
  ModBan(ModBanView),
  ModBanFromCommunity(ModBanFromCommunityView),
  ModFeaturePost(ModFeaturePostView),
  ModHideCommunity(ModHideCommunityView),
  ModLockPost(ModLockPostView),
  ModRemoveComment(ModRemoveCommentView),
  ModRemoveCommunity(ModRemoveCommunityView),
  ModRemovePost(ModRemovePostView),
  ModTransferCommunity(ModTransferCommunityView),
}
