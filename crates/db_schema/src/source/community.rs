#[cfg(feature = "full")]
use crate::schema::{community, community_actions};
use crate::{
  newtypes::{CommunityId, DbUrl, InstanceId, PersonId},
  sensitive::SensitiveString,
  source::placeholder_apub_url,
  CommunityVisibility,
};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel::{dsl, expression_methods::NullableExplessionMethods};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = community))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A community.
pub struct Community {
  pub id: CommunityId,
  pub name: String,
  /// A longer title, that can contain other characters, and doesn't have to be unique.
  pub title: String,
  /// A sidebar / markdown description.
  pub description: Option<String>,
  /// Whether the community is removed by a mod.
  pub removed: bool,
  pub published: DateTime<Utc>,
  pub updated: Option<DateTime<Utc>>,
  /// Whether the community has been deleted by its creator.
  pub deleted: bool,
  /// Whether its an NSFW community.
  pub nsfw: bool,
  /// The federated actor_id.
  pub actor_id: DbUrl,
  /// Whether the community is local.
  pub local: bool,
  #[serde(skip)]
  pub private_key: Option<SensitiveString>,
  #[serde(skip)]
  pub public_key: String,
  #[serde(skip)]
  pub last_refreshed_at: DateTime<Utc>,
  /// A URL for an icon.
  pub icon: Option<DbUrl>,
  /// A URL for a banner.
  pub banner: Option<DbUrl>,
  #[cfg_attr(feature = "full", ts(skip))]
  #[serde(skip)]
  pub followers_url: Option<DbUrl>,
  #[cfg_attr(feature = "full", ts(skip))]
  #[serde(skip, default = "placeholder_apub_url")]
  pub inbox_url: DbUrl,
  #[serde(skip)]
  pub shared_inbox_url: Option<DbUrl>,
  /// Whether the community is hidden.
  pub hidden: bool,
  /// Whether posting is restricted to mods only.
  pub posting_restricted_to_mods: bool,
  pub instance_id: InstanceId,
  /// Url where moderators collection is served over Activitypub
  #[serde(skip)]
  pub moderators_url: Option<DbUrl>,
  /// Url where featured posts collection is served over Activitypub
  #[serde(skip)]
  pub featured_url: Option<DbUrl>,
  pub visibility: CommunityVisibility,
}

#[derive(Debug, Clone, TypedBuilder, Default)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community))]
pub struct CommunityInsertForm {
  #[builder(!default)]
  pub name: String,
  #[builder(!default)]
  pub title: String,
  pub description: Option<String>,
  pub removed: Option<bool>,
  pub published: Option<DateTime<Utc>>,
  pub updated: Option<DateTime<Utc>>,
  pub deleted: Option<bool>,
  pub nsfw: Option<bool>,
  pub actor_id: Option<DbUrl>,
  pub local: Option<bool>,
  pub private_key: Option<String>,
  pub public_key: String,
  pub last_refreshed_at: Option<DateTime<Utc>>,
  pub icon: Option<DbUrl>,
  pub banner: Option<DbUrl>,
  pub followers_url: Option<DbUrl>,
  pub inbox_url: Option<DbUrl>,
  pub shared_inbox_url: Option<DbUrl>,
  pub moderators_url: Option<DbUrl>,
  pub featured_url: Option<DbUrl>,
  pub hidden: Option<bool>,
  pub posting_restricted_to_mods: Option<bool>,
  #[builder(!default)]
  pub instance_id: InstanceId,
  pub visibility: Option<CommunityVisibility>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community))]
pub struct CommunityUpdateForm {
  pub title: Option<String>,
  pub description: Option<Option<String>>,
  pub removed: Option<bool>,
  pub published: Option<DateTime<Utc>>,
  pub updated: Option<Option<DateTime<Utc>>>,
  pub deleted: Option<bool>,
  pub nsfw: Option<bool>,
  pub actor_id: Option<DbUrl>,
  pub local: Option<bool>,
  pub public_key: Option<String>,
  pub private_key: Option<Option<String>>,
  pub last_refreshed_at: Option<DateTime<Utc>>,
  pub icon: Option<Option<DbUrl>>,
  pub banner: Option<Option<DbUrl>>,
  pub followers_url: Option<DbUrl>,
  pub inbox_url: Option<DbUrl>,
  pub shared_inbox_url: Option<Option<DbUrl>>,
  pub moderators_url: Option<DbUrl>,
  pub featured_url: Option<DbUrl>,
  pub hidden: Option<bool>,
  pub posting_restricted_to_mods: Option<bool>,
  pub visibility: Option<CommunityVisibility>,
}

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::community::Community))
)]
#[cfg_attr(feature = "full", diesel(table_name = community_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, community_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct CommunityModerator {
  pub community_id: CommunityId,
  pub person_id: PersonId,
  #[cfg_attr(feature = "full", diesel(select_expression = community_actions::became_moderator.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<community_actions::became_moderator>))]
  pub published: DateTime<Utc>,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community_actions))]
pub struct CommunityModeratorForm {
  pub community_id: CommunityId,
  pub person_id: PersonId,
}

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::community::Community))
)]
#[cfg_attr(feature = "full", diesel(table_name = community_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, community_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct CommunityPersonBan {
  pub community_id: CommunityId,
  pub person_id: PersonId,
  #[cfg_attr(feature = "full", diesel(select_expression = community_actions::received_ban.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<community_actions::received_ban>))]
  pub published: DateTime<Utc>,
  #[cfg_attr(feature = "full", diesel(select_expression = community_actions::ban_expires.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<community_actions::ban_expires>))]
  pub expires: Option<DateTime<Utc>>,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community_actions))]
pub struct CommunityPersonBanForm {
  pub community_id: CommunityId,
  pub person_id: PersonId,
  #[cfg_attr(feature = "full", diesel(column_name = ban_expires))]
  pub expires: Option<Option<DateTime<Utc>>>,
}

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::community::Community))
)]
#[cfg_attr(feature = "full", diesel(table_name = community_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, community_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct CommunityFollower {
  pub community_id: CommunityId,
  pub person_id: PersonId,
  #[cfg_attr(feature = "full", diesel(select_expression = community_actions::followed.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<community_actions::followed>))]
  pub published: DateTime<Utc>,
  #[cfg_attr(feature = "full", diesel(select_expression = community_actions::follow_pending.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<community_actions::follow_pending>))]
  pub pending: bool,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community_actions))]
pub struct CommunityFollowerForm {
  pub community_id: CommunityId,
  pub person_id: PersonId,
  #[cfg_attr(feature = "full", diesel(column_name = follow_pending))]
  pub pending: bool,
}
