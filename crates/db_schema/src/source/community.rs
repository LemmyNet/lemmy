use crate::newtypes::{CommunityId, DbUrl, PersonId};
use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
use crate::schema::{community, community_follower, community_moderator, community_person_ban};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", table_name = "community")]
pub struct Community {
  pub id: CommunityId,
  pub name: String,
  pub title: String,
  pub description: Option<String>,
  pub removed: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub nsfw: bool,
  pub actor_id: DbUrl,
  pub local: bool,
  pub private_key: Option<String>,
  pub public_key: String,
  pub last_refreshed_at: chrono::NaiveDateTime,
  pub icon: Option<DbUrl>,
  pub banner: Option<DbUrl>,
  pub followers_url: DbUrl,
  pub inbox_url: DbUrl,
  pub shared_inbox_url: Option<DbUrl>,
  pub hidden: bool,
  pub posting_restricted_to_mods: bool,
}

/// A safe representation of community, without the sensitive info
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", table_name = "community")]
pub struct CommunitySafe {
  pub id: CommunityId,
  pub name: String,
  pub title: String,
  pub description: Option<String>,
  pub removed: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub nsfw: bool,
  pub actor_id: DbUrl,
  pub local: bool,
  pub icon: Option<DbUrl>,
  pub banner: Option<DbUrl>,
  pub hidden: bool,
  pub posting_restricted_to_mods: bool,
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", table_name = "community")]
pub struct CommunityForm {
  pub name: String,
  pub title: String,
  pub description: Option<Option<String>>,
  pub removed: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: Option<bool>,
  pub nsfw: Option<bool>,
  pub actor_id: Option<DbUrl>,
  pub local: Option<bool>,
  pub private_key: Option<Option<String>>,
  pub public_key: Option<String>,
  pub last_refreshed_at: Option<chrono::NaiveDateTime>,
  pub icon: Option<Option<DbUrl>>,
  pub banner: Option<Option<DbUrl>>,
  pub followers_url: Option<DbUrl>,
  pub inbox_url: Option<DbUrl>,
  pub shared_inbox_url: Option<Option<DbUrl>>,
  pub hidden: Option<bool>,
  pub posting_restricted_to_mods: Option<bool>,
}

#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Associations))]
#[cfg_attr(feature = "full", belongs_to(Community))]
#[cfg_attr(feature = "full", table_name = "community_moderator")]
pub struct CommunityModerator {
  pub id: i32,
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub published: chrono::NaiveDateTime,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", table_name = "community_moderator")]
pub struct CommunityModeratorForm {
  pub community_id: CommunityId,
  pub person_id: PersonId,
}

#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Associations))]
#[cfg_attr(feature = "full", belongs_to(Community))]
#[cfg_attr(feature = "full", table_name = "community_person_ban")]
pub struct CommunityPersonBan {
  pub id: i32,
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub published: chrono::NaiveDateTime,
  pub expires: Option<chrono::NaiveDateTime>,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", table_name = "community_person_ban")]
pub struct CommunityPersonBanForm {
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub expires: Option<Option<chrono::NaiveDateTime>>,
}

#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Associations))]
#[cfg_attr(feature = "full", belongs_to(Community))]
#[cfg_attr(feature = "full", table_name = "community_follower")]
pub struct CommunityFollower {
  pub id: i32,
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub published: chrono::NaiveDateTime,
  pub pending: Option<bool>,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", table_name = "community_follower")]
pub struct CommunityFollowerForm {
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub pending: bool,
}
