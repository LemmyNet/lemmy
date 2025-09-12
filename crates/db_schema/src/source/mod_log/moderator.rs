use crate::newtypes::{
  CommentId,
  CommunityId,
  ModAddToCommunityId,
  ModBanFromCommunityId,
  ModChangeCommunityVisibilityId,
  ModFeaturePostId,
  ModLockCommentId,
  ModLockPostId,
  ModRemoveCommentId,
  ModRemovePostId,
  ModTransferCommunityId,
  PersonId,
  PostId,
};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::CommunityVisibility;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::{
  mod_add_to_community,
  mod_ban_from_community,
  mod_change_community_visibility,
  mod_feature_post,
  mod_lock_comment,
  mod_lock_post,
  mod_remove_comment,
  mod_remove_post,
  mod_transfer_community,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_remove_post))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator removes a post.
pub struct ModRemovePost {
  pub id: ModRemovePostId,
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub reason: Option<String>,
  pub removed: bool,
  pub published_at: DateTime<Utc>,
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
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_lock_post))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator locks a post (prevents new comments being made).
pub struct ModLockPost {
  pub id: ModLockPostId,
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub locked: bool,
  pub published_at: DateTime<Utc>,
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
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_feature_post))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator features a post on a community (pins it to the top).
pub struct ModFeaturePost {
  pub id: ModFeaturePostId,
  pub mod_person_id: PersonId,
  pub post_id: PostId,
  pub featured: bool,
  pub published_at: DateTime<Utc>,
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
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_remove_comment))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator removes a comment.
pub struct ModRemoveComment {
  pub id: ModRemoveCommentId,
  pub mod_person_id: PersonId,
  pub comment_id: CommentId,
  pub reason: Option<String>,
  pub removed: bool,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_remove_comment))]
pub struct ModRemoveCommentForm {
  pub mod_person_id: PersonId,
  pub comment_id: CommentId,
  pub reason: Option<String>,
  pub removed: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_lock_comment))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator locks a comment (prevents new replies to a comment or its children).
pub struct ModLockComment {
  pub id: ModLockCommentId,
  pub mod_person_id: PersonId,
  pub comment_id: CommentId,
  pub locked: bool,
  pub reason: Option<String>,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_lock_comment))]
pub struct ModLockCommentForm {
  pub mod_person_id: PersonId,
  pub comment_id: CommentId,
  pub locked: Option<bool>,
  pub reason: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_ban_from_community))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When someone is banned from a community.
pub struct ModBanFromCommunity {
  pub id: ModBanFromCommunityId,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub community_id: CommunityId,
  pub reason: Option<String>,
  pub banned: bool,
  pub expires_at: Option<DateTime<Utc>>,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_ban_from_community))]
pub struct ModBanFromCommunityForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub community_id: CommunityId,
  pub reason: Option<String>,
  pub banned: Option<bool>,
  pub expires_at: Option<DateTime<Utc>>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_change_community_visibility))]
pub struct ModChangeCommunityVisibilityForm {
  pub community_id: CommunityId,
  pub mod_person_id: PersonId,
  pub visibility: CommunityVisibility,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_change_community_visibility))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ModChangeCommunityVisibility {
  pub id: ModChangeCommunityVisibilityId,
  pub community_id: CommunityId,
  pub mod_person_id: PersonId,
  pub published_at: DateTime<Utc>,
  pub visibility: CommunityVisibility,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_add_to_community))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When someone is added as a community moderator.
pub struct ModAddToCommunity {
  pub id: ModAddToCommunityId,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub community_id: CommunityId,
  pub removed: bool,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_add_to_community))]
pub struct ModAddToCommunityForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub community_id: CommunityId,
  pub removed: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = mod_transfer_community))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator transfers a community to a new owner.
pub struct ModTransferCommunity {
  pub id: ModTransferCommunityId,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub community_id: CommunityId,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = mod_transfer_community))]
pub struct ModTransferCommunityForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub community_id: CommunityId,
}
