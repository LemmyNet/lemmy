use crate::newtypes::{CommentId, CommunityId, PersonId, PostId};
#[cfg(feature = "full")]
use crate::schema::{
  admin_purge_comment,
  admin_purge_community,
  admin_purge_person,
  admin_purge_post,
  mod_add,
  mod_add_community,
  mod_ban,
  mod_ban_from_community,
  mod_feature_post,
  mod_hide_community,
  mod_lock_post,
  mod_remove_comment,
  mod_remove_community,
  mod_remove_post,
  mod_transfer_community,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = mod_remove_post))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator removes a post.
pub struct ModRemovePost {
  pub id: i32,
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub reason: Option<String>,
  pub removed: bool,
  pub when_: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_remove_post))]
pub struct ModRemovePostForm {
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub reason: Option<String>,
  pub removed: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = mod_lock_post))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator locks a post (prevents new comments being made).
pub struct ModLockPost {
  pub id: i32,
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub locked: bool,
  pub when_: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_lock_post))]
pub struct ModLockPostForm {
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub locked: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = mod_feature_post))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator features a post on a community (pins it to the top).
pub struct ModFeaturePost {
  pub id: i32,
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub featured: bool,
  pub when_: DateTime<Utc>,
  pub is_featured_community: bool,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_feature_post))]
pub struct ModFeaturePostForm {
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub featured: bool,
  pub is_featured_community: bool,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = mod_remove_comment))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator removes a comment.
pub struct ModRemoveComment {
  pub id: i32,
  pub mod_person_id: PersonId,
  pub comment_id: CommentId,
  pub reason: Option<String>,
  pub removed: bool,
  pub when_: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_remove_comment))]
pub struct ModRemoveCommentForm {
  pub mod_person_id: PersonId,
  pub comment_id: CommentId,
  pub reason: Option<String>,
  pub removed: Option<bool>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = mod_remove_community))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator removes a community.
pub struct ModRemoveCommunity {
  pub id: i32,
  pub mod_person_id: PersonId,
  pub community_id: CommunityId,
  pub reason: Option<String>,
  pub removed: bool,
  pub when_: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_remove_community))]
pub struct ModRemoveCommunityForm {
  pub mod_person_id: PersonId,
  pub community_id: CommunityId,
  pub reason: Option<String>,
  pub removed: Option<bool>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = mod_ban_from_community))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When someone is banned from a community.
pub struct ModBanFromCommunity {
  pub id: i32,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub community_id: CommunityId,
  pub reason: Option<String>,
  pub banned: bool,
  pub expires: Option<DateTime<Utc>>,
  pub when_: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_ban_from_community))]
pub struct ModBanFromCommunityForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub community_id: CommunityId,
  pub reason: Option<String>,
  pub banned: Option<bool>,
  pub expires: Option<DateTime<Utc>>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = mod_ban))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When someone is banned from the site.
pub struct ModBan {
  pub id: i32,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub reason: Option<String>,
  pub banned: bool,
  pub expires: Option<DateTime<Utc>>,
  pub when_: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_hide_community))]
pub struct ModHideCommunityForm {
  pub community_id: CommunityId,
  pub mod_person_id: PersonId,
  pub hidden: Option<bool>,
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = mod_hide_community))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a community is hidden from public view.
pub struct ModHideCommunity {
  pub id: i32,
  pub community_id: CommunityId,
  pub mod_person_id: PersonId,
  pub when_: DateTime<Utc>,
  pub reason: Option<String>,
  pub hidden: bool,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_ban))]
pub struct ModBanForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub reason: Option<String>,
  pub banned: Option<bool>,
  pub expires: Option<DateTime<Utc>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = mod_add_community))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When someone is added as a community moderator.
pub struct ModAddCommunity {
  pub id: i32,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub community_id: CommunityId,
  pub removed: bool,
  pub when_: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_add_community))]
pub struct ModAddCommunityForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub community_id: CommunityId,
  pub removed: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = mod_transfer_community))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator transfers a community to a new owner.
pub struct ModTransferCommunity {
  pub id: i32,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub community_id: CommunityId,
  pub when_: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_transfer_community))]
pub struct ModTransferCommunityForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub community_id: CommunityId,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = mod_add))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When someone is added as a site moderator.
pub struct ModAdd {
  pub id: i32,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub removed: bool,
  pub when_: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_add))]
pub struct ModAddForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub removed: Option<bool>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = admin_purge_person))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a person.
pub struct AdminPurgePerson {
  pub id: i32,
  pub admin_person_id: PersonId,
  pub reason: Option<String>,
  pub when_: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_purge_person))]
pub struct AdminPurgePersonForm {
  pub admin_person_id: PersonId,
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = admin_purge_community))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a community.
pub struct AdminPurgeCommunity {
  pub id: i32,
  pub admin_person_id: PersonId,
  pub reason: Option<String>,
  pub when_: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_purge_community))]
pub struct AdminPurgeCommunityForm {
  pub admin_person_id: PersonId,
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = admin_purge_post))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a post.
pub struct AdminPurgePost {
  pub id: i32,
  pub admin_person_id: PersonId,
  pub community_id: CommunityId,
  pub reason: Option<String>,
  pub when_: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_purge_post))]
pub struct AdminPurgePostForm {
  pub admin_person_id: PersonId,
  pub community_id: CommunityId,
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = admin_purge_comment))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When an admin purges a comment.
pub struct AdminPurgeComment {
  pub id: i32,
  pub admin_person_id: PersonId,
  pub post_id: PostId,
  pub reason: Option<String>,
  pub when_: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_purge_comment))]
pub struct AdminPurgeCommentForm {
  pub admin_person_id: PersonId,
  pub post_id: PostId,
  pub reason: Option<String>,
}
