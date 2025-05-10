use crate::newtypes::{
  CommentId,
  CommunityId,
  InstanceId,
  ModAddCommunityId,
  ModAddId,
  ModBanFromCommunityId,
  ModBanId,
  ModChangeCommunityVisibilityId,
  ModFeaturePostId,
  ModLockPostId,
  ModRemoveCommentId,
  ModRemoveCommunityId,
  ModRemovePostId,
  ModTransferCommunityId,
  PersonId,
  PostId,
};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::CommunityVisibility;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::{
  mod_add,
  mod_add_community,
  mod_ban,
  mod_ban_from_community,
  mod_change_community_visibility,
  mod_feature_post,
  mod_lock_post,
  mod_remove_comment,
  mod_remove_community,
  mod_remove_post,
  mod_transfer_community,
};
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
  pub id: ModRemovePostId,
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
  pub removed: bool,
  pub published: DateTime<Utc>,
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
  pub id: ModLockPostId,
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub locked: bool,
  pub published: DateTime<Utc>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_lock_post))]
pub struct ModLockPostForm {
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub locked: Option<bool>,
  pub reason: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = mod_feature_post))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator features a post on a community (pins it to the top).
pub struct ModFeaturePost {
  pub id: ModFeaturePostId,
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub featured: bool,
  pub published: DateTime<Utc>,
  pub is_featured_community: bool,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_feature_post))]
pub struct ModFeaturePostForm {
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub featured: Option<bool>,
  pub is_featured_community: Option<bool>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = mod_remove_comment))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When a moderator removes a comment.
pub struct ModRemoveComment {
  pub id: ModRemoveCommentId,
  pub mod_person_id: PersonId,
  pub comment_id: CommentId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
  pub removed: bool,
  pub published: DateTime<Utc>,
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
  pub id: ModRemoveCommunityId,
  pub mod_person_id: PersonId,
  pub community_id: CommunityId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
  pub removed: bool,
  pub published: DateTime<Utc>,
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
  pub id: ModBanFromCommunityId,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub community_id: CommunityId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
  pub banned: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub expires: Option<DateTime<Utc>>,
  pub published: DateTime<Utc>,
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
  pub id: ModBanId,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
  pub banned: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub expires: Option<DateTime<Utc>>,
  pub published: DateTime<Utc>,
  pub instance_id: InstanceId,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_change_community_visibility))]
pub struct ModChangeCommunityVisibilityForm {
  pub community_id: CommunityId,
  pub mod_person_id: PersonId,
  pub reason: Option<String>,
  pub visibility: CommunityVisibility,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = mod_change_community_visibility))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ModChangeCommunityVisibility {
  pub id: ModChangeCommunityVisibilityId,
  pub community_id: CommunityId,
  pub mod_person_id: PersonId,
  pub published: DateTime<Utc>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
  pub visibility: CommunityVisibility,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_ban))]
pub struct ModBanForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub reason: Option<String>,
  pub banned: Option<bool>,
  pub expires: Option<DateTime<Utc>>,
  pub instance_id: InstanceId,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = mod_add_community))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// When someone is added as a community moderator.
pub struct ModAddCommunity {
  pub id: ModAddCommunityId,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub community_id: CommunityId,
  pub removed: bool,
  pub published: DateTime<Utc>,
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
  pub id: ModTransferCommunityId,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub community_id: CommunityId,
  pub published: DateTime<Utc>,
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
  pub id: ModAddId,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub removed: bool,
  pub published: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_add))]
pub struct ModAddForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub removed: Option<bool>,
}
