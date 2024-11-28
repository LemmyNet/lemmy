use crate::newtypes::{CommunityId, InstanceId, PersonId, PostId};
#[cfg(feature = "full")]
use crate::schema::{
  admin_allow_instance,
  admin_block_instance,
  admin_purge_comment,
  admin_purge_community,
  admin_purge_person,
  admin_purge_post,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

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
  #[cfg_attr(feature = "full", ts(optional))]
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
  #[cfg_attr(feature = "full", ts(optional))]
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
  #[cfg_attr(feature = "full", ts(optional))]
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
  #[cfg_attr(feature = "full", ts(optional))]
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

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(TS, Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::instance::Instance))
)]
#[cfg_attr(feature = "full", diesel(table_name = admin_allow_instance))]
#[cfg_attr(feature = "full", diesel(primary_key(instance_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AdminAllowInstance {
  pub id: i32,
  pub instance_id: InstanceId,
  pub admin_person_id: PersonId,
  pub allowed: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
  pub when_: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_allow_instance))]
pub struct AdminAllowInstanceForm {
  pub instance_id: InstanceId,
  pub admin_person_id: PersonId,
  pub allowed: bool,
  pub reason: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(TS, Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::instance::Instance))
)]
#[cfg_attr(feature = "full", diesel(table_name = admin_block_instance))]
#[cfg_attr(feature = "full", diesel(primary_key(instance_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct AdminBlockInstance {
  pub id: i32,
  pub instance_id: InstanceId,
  pub admin_person_id: PersonId,
  pub blocked: bool,
  #[cfg_attr(feature = "full", ts(optional))]
  pub reason: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub expires: Option<DateTime<Utc>>,
  pub when_: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = admin_block_instance))]
pub struct AdminBlockInstanceForm {
  pub instance_id: InstanceId,
  pub admin_person_id: PersonId,
  pub blocked: bool,
  pub reason: Option<String>,
  pub when_: Option<DateTime<Utc>>,
}
