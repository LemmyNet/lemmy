#[cfg(feature = "full")]
use diesel::Queryable;
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, PersonId, PostId},
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
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When someone is added as a community moderator.
pub struct ModAddCommunityView {
  pub mod_add_community: ModAddCommunity,
  #[ts(optional)]
  pub moderator: Option<Person>,
  pub community: Community,
  pub modded_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When someone is added as a site moderator.
pub struct ModAddView {
  pub mod_add: ModAdd,
  #[ts(optional)]
  pub moderator: Option<Person>,
  pub modded_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When someone is banned from a community.
pub struct ModBanFromCommunityView {
  pub mod_ban_from_community: ModBanFromCommunity,
  #[ts(optional)]
  pub moderator: Option<Person>,
  pub community: Community,
  pub banned_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When someone is banned from the site.
pub struct ModBanView {
  pub mod_ban: ModBan,
  #[ts(optional)]
  pub moderator: Option<Person>,
  pub banned_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a community is hidden from public view.
pub struct ModHideCommunityView {
  pub mod_hide_community: ModHideCommunity,
  #[ts(optional)]
  pub admin: Option<Person>,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator locks a post (prevents new comments being made).
pub struct ModLockPostView {
  pub mod_lock_post: ModLockPost,
  #[ts(optional)]
  pub moderator: Option<Person>,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator removes a comment.
pub struct ModRemoveCommentView {
  pub mod_remove_comment: ModRemoveComment,
  #[ts(optional)]
  pub moderator: Option<Person>,
  pub comment: Comment,
  pub commenter: Person,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator removes a community.
pub struct ModRemoveCommunityView {
  pub mod_remove_community: ModRemoveCommunity,
  #[ts(optional)]
  pub moderator: Option<Person>,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator removes a post.
pub struct ModRemovePostView {
  pub mod_remove_post: ModRemovePost,
  #[ts(optional)]
  pub moderator: Option<Person>,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator features a post on a community (pins it to the top).
pub struct ModFeaturePostView {
  pub mod_feature_post: ModFeaturePost,
  #[ts(optional)]
  pub moderator: Option<Person>,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator transfers a community to a new owner.
pub struct ModTransferCommunityView {
  pub mod_transfer_community: ModTransferCommunity,
  #[ts(optional)]
  pub moderator: Option<Person>,
  pub community: Community,
  pub modded_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a comment.
pub struct AdminPurgeCommentView {
  pub admin_purge_comment: AdminPurgeComment,
  #[ts(optional)]
  pub admin: Option<Person>,
  pub post: Post,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a community.
pub struct AdminPurgeCommunityView {
  pub admin_purge_community: AdminPurgeCommunity,
  #[ts(optional)]
  pub admin: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a person.
pub struct AdminPurgePersonView {
  pub admin_purge_person: AdminPurgePerson,
  #[ts(optional)]
  pub admin: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a post.
pub struct AdminPurgePostView {
  pub admin_purge_post: AdminPurgePost,
  #[ts(optional)]
  pub admin: Option<Person>,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// Querying / filtering the modlog.
pub struct ModlogListParams {
  #[ts(optional)]
  pub community_id: Option<CommunityId>,
  #[ts(optional)]
  pub mod_person_id: Option<PersonId>,
  #[ts(optional)]
  pub other_person_id: Option<PersonId>,
  #[ts(optional)]
  pub post_id: Option<PostId>,
  #[ts(optional)]
  pub comment_id: Option<CommentId>,
  #[ts(optional)]
  pub page: Option<i64>,
  #[ts(optional)]
  pub limit: Option<i64>,
  pub hide_modlog_names: bool,
}
