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
use lemmy_proc_macros::lemmy_dto;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[lemmy_dto]
/// When someone is added as a community moderator.
pub struct ModAddCommunityView {
  pub mod_add_community: ModAddCommunity,
  pub moderator: Option<Person>,
  pub community: Community,
  pub modded_person: Person,
}

#[lemmy_dto]
/// When someone is added as a site moderator.
pub struct ModAddView {
  pub mod_add: ModAdd,
  pub moderator: Option<Person>,
  pub modded_person: Person,
}

#[lemmy_dto]
/// When someone is banned from a community.
pub struct ModBanFromCommunityView {
  pub mod_ban_from_community: ModBanFromCommunity,
  pub moderator: Option<Person>,
  pub community: Community,
  pub banned_person: Person,
}

#[lemmy_dto]
/// When someone is banned from the site.
pub struct ModBanView {
  pub mod_ban: ModBan,
  pub moderator: Option<Person>,
  pub banned_person: Person,
}

#[lemmy_dto]
/// When a community is hidden from public view.
pub struct ModHideCommunityView {
  pub mod_hide_community: ModHideCommunity,
  pub admin: Option<Person>,
  pub community: Community,
}

#[lemmy_dto]
/// When a moderator locks a post (prevents new comments being made).
pub struct ModLockPostView {
  pub mod_lock_post: ModLockPost,
  pub moderator: Option<Person>,
  pub post: Post,
  pub community: Community,
}

#[lemmy_dto]
/// When a moderator removes a comment.
pub struct ModRemoveCommentView {
  pub mod_remove_comment: ModRemoveComment,
  pub moderator: Option<Person>,
  pub comment: Comment,
  pub commenter: Person,
  pub post: Post,
  pub community: Community,
}

#[lemmy_dto]
/// When a moderator removes a community.
pub struct ModRemoveCommunityView {
  pub mod_remove_community: ModRemoveCommunity,
  pub moderator: Option<Person>,
  pub community: Community,
}

#[lemmy_dto]
/// When a moderator removes a post.
pub struct ModRemovePostView {
  pub mod_remove_post: ModRemovePost,
  pub moderator: Option<Person>,
  pub post: Post,
  pub community: Community,
}

#[lemmy_dto]
/// When a moderator features a post on a community (pins it to the top).
pub struct ModFeaturePostView {
  pub mod_feature_post: ModFeaturePost,
  pub moderator: Option<Person>,
  pub post: Post,
  pub community: Community,
}

#[lemmy_dto]
/// When a moderator transfers a community to a new owner.
pub struct ModTransferCommunityView {
  pub mod_transfer_community: ModTransferCommunity,
  pub moderator: Option<Person>,
  pub community: Community,
  pub modded_person: Person,
}

#[lemmy_dto]
/// When an admin purges a comment.
pub struct AdminPurgeCommentView {
  pub admin_purge_comment: AdminPurgeComment,
  pub admin: Option<Person>,
  pub post: Post,
}

#[lemmy_dto]
/// When an admin purges a community.
pub struct AdminPurgeCommunityView {
  pub admin_purge_community: AdminPurgeCommunity,
  pub admin: Option<Person>,
}

#[lemmy_dto]
/// When an admin purges a person.
pub struct AdminPurgePersonView {
  pub admin_purge_person: AdminPurgePerson,
  pub admin: Option<Person>,
}

#[lemmy_dto]
/// When an admin purges a post.
pub struct AdminPurgePostView {
  pub admin_purge_post: AdminPurgePost,
  pub admin: Option<Person>,
  pub community: Community,
}

#[lemmy_dto(Copy)]
/// Querying / filtering the modlog.
pub struct ModlogListParams {
  pub community_id: Option<CommunityId>,
  pub mod_person_id: Option<PersonId>,
  pub other_person_id: Option<PersonId>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub hide_modlog_names: bool,
}
