use crate::newtypes::{
  AdminAddId,
  AdminAllowInstanceId,
  AdminBanId,
  AdminBlockInstanceId,
  AdminPurgeCommentId,
  AdminPurgeCommunityId,
  AdminPurgePersonId,
  AdminPurgePostId,
  AdminRemoveCommunityId,
  CommunityId,
  InstanceId,
  PersonId,
  PostId,
};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::{
  admin_add,
  admin_allow_instance,
  admin_ban,
  admin_block_instance,
  admin_purge_comment,
  admin_purge_community,
  admin_purge_person,
  admin_purge_post,
  admin_remove_community,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = admin_purge_person))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin purges a person.
pub struct AdminPurgePerson {
  pub id: AdminPurgePersonId,
  pub admin_person_id: PersonId,
  pub reason: String,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_purge_person))]
pub struct AdminPurgePersonForm {
  pub admin_person_id: PersonId,
  pub reason: String,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = admin_purge_community))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin purges a community.
pub struct AdminPurgeCommunity {
  pub id: AdminPurgeCommunityId,
  pub admin_person_id: PersonId,
  pub reason: String,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_purge_community))]
pub struct AdminPurgeCommunityForm {
  pub admin_person_id: PersonId,
  pub reason: String,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = admin_purge_post))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin purges a post.
pub struct AdminPurgePost {
  pub id: AdminPurgePostId,
  pub admin_person_id: PersonId,
  pub community_id: CommunityId,
  pub reason: String,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_purge_post))]
pub struct AdminPurgePostForm {
  pub admin_person_id: PersonId,
  pub community_id: CommunityId,
  pub reason: String,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = admin_purge_comment))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin purges a comment.
pub struct AdminPurgeComment {
  pub id: AdminPurgeCommentId,
  pub admin_person_id: PersonId,
  pub post_id: PostId,
  pub reason: String,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_purge_comment))]
pub struct AdminPurgeCommentForm {
  pub admin_person_id: PersonId,
  pub post_id: PostId,
  pub reason: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::instance::Instance))
)]
#[cfg_attr(feature = "full", diesel(table_name = admin_allow_instance))]
#[cfg_attr(feature = "full", diesel(primary_key(instance_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminAllowInstance {
  pub id: AdminAllowInstanceId,
  pub instance_id: InstanceId,
  pub admin_person_id: PersonId,
  pub allowed: bool,
  pub reason: String,
  pub published_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_allow_instance))]
pub struct AdminAllowInstanceForm {
  pub instance_id: InstanceId,
  pub admin_person_id: PersonId,
  pub allowed: bool,
  pub reason: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::instance::Instance))
)]
#[cfg_attr(feature = "full", diesel(table_name = admin_block_instance))]
#[cfg_attr(feature = "full", diesel(primary_key(instance_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct AdminBlockInstance {
  pub id: AdminBlockInstanceId,
  pub instance_id: InstanceId,
  pub admin_person_id: PersonId,
  pub blocked: bool,
  pub reason: String,
  pub expires_at: Option<DateTime<Utc>>,
  pub published_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_block_instance))]
pub struct AdminBlockInstanceForm {
  pub instance_id: InstanceId,
  pub admin_person_id: PersonId,
  pub blocked: bool,
  pub reason: String,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = admin_remove_community))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator removes a community.
pub struct AdminRemoveCommunity {
  pub id: AdminRemoveCommunityId,
  pub mod_person_id: PersonId,
  pub community_id: CommunityId,
  pub reason: String,
  pub removed: bool,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_remove_community))]
pub struct AdminRemoveCommunityForm {
  pub mod_person_id: PersonId,
  pub community_id: CommunityId,
  pub reason: String,
  pub removed: Option<bool>,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = admin_ban))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When someone is banned from the site.
pub struct AdminBan {
  pub id: AdminBanId,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub reason: String,
  pub banned: bool,
  pub expires_at: Option<DateTime<Utc>>,
  pub published_at: DateTime<Utc>,
  pub instance_id: InstanceId,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_ban))]
pub struct AdminBanForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub reason: String,
  pub banned: Option<bool>,
  pub expires_at: Option<DateTime<Utc>>,
  pub instance_id: InstanceId,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = admin_add))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When someone is added as a site admin.
pub struct AdminAdd {
  pub id: AdminAddId,
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub removed: bool,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_add))]
pub struct AdminAddForm {
  pub mod_person_id: PersonId,
  pub other_person_id: PersonId,
  pub removed: Option<bool>,
}
