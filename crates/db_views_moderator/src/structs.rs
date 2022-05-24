use lemmy_db_schema::source::{
  comment::Comment,
  community::CommunitySafe,
  moderator::{
    AdminPurgeComment,
    AdminPurgeCommunity,
    AdminPurgePerson,
    AdminPurgePost,
    ModAdd,
    ModAddCommunity,
    ModBan,
    ModBanFromCommunity,
    ModHideCommunity,
    ModLockPost,
    ModRemoveComment,
    ModRemoveCommunity,
    ModRemovePost,
    ModStickyPost,
    ModTransferCommunity,
  },
  person::{PersonSafe, PersonSafeAlias1},
  post::Post,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModAddCommunityView {
  pub mod_add_community: ModAddCommunity,
  pub moderator: PersonSafe,
  pub community: CommunitySafe,
  pub modded_person: PersonSafeAlias1,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModAddView {
  pub mod_add: ModAdd,
  pub moderator: PersonSafe,
  pub modded_person: PersonSafeAlias1,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModBanFromCommunityView {
  pub mod_ban_from_community: ModBanFromCommunity,
  pub moderator: PersonSafe,
  pub community: CommunitySafe,
  pub banned_person: PersonSafeAlias1,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModBanView {
  pub mod_ban: ModBan,
  pub moderator: PersonSafe,
  pub banned_person: PersonSafeAlias1,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModHideCommunityView {
  pub mod_hide_community: ModHideCommunity,
  pub admin: PersonSafe,
  pub community: CommunitySafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModLockPostView {
  pub mod_lock_post: ModLockPost,
  pub moderator: PersonSafe,
  pub post: Post,
  pub community: CommunitySafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModRemoveCommentView {
  pub mod_remove_comment: ModRemoveComment,
  pub moderator: PersonSafe,
  pub comment: Comment,
  pub commenter: PersonSafeAlias1,
  pub post: Post,
  pub community: CommunitySafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModRemoveCommunityView {
  pub mod_remove_community: ModRemoveCommunity,
  pub moderator: PersonSafe,
  pub community: CommunitySafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModRemovePostView {
  pub mod_remove_post: ModRemovePost,
  pub moderator: PersonSafe,
  pub post: Post,
  pub community: CommunitySafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModStickyPostView {
  pub mod_sticky_post: ModStickyPost,
  pub moderator: PersonSafe,
  pub post: Post,
  pub community: CommunitySafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModTransferCommunityView {
  pub mod_transfer_community: ModTransferCommunity,
  pub moderator: PersonSafe,
  pub community: CommunitySafe,
  pub modded_person: PersonSafeAlias1,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminPurgeCommentView {
  pub admin_purge_comment: AdminPurgeComment,
  pub admin: PersonSafe,
  pub post: Post,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminPurgeCommunityView {
  pub admin_purge_community: AdminPurgeCommunity,
  pub admin: PersonSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminPurgePersonView {
  pub admin_purge_person: AdminPurgePerson,
  pub admin: PersonSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminPurgePostView {
  pub admin_purge_post: AdminPurgePost,
  pub admin: PersonSafe,
  pub community: CommunitySafe,
}
