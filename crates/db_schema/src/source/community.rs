use crate::{
  schema::{community, community_follower, community_moderator, community_person_ban},
  CommunityId,
  DbUrl,
  PersonId,
};
use serde::Serialize;

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "community"]
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
  pub public_key: Option<String>,
  pub last_refreshed_at: chrono::NaiveDateTime,
  pub icon: Option<DbUrl>,
  pub banner: Option<DbUrl>,
  pub followers_url: DbUrl,
  pub inbox_url: DbUrl,
  pub shared_inbox_url: Option<DbUrl>,
}

/// A safe representation of community, without the sensitive info
#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "community"]
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
}

#[derive(Insertable, AsChangeset, Debug, Default)]
#[table_name = "community"]
pub struct CommunityForm {
  pub name: String,
  pub title: String,
  pub description: Option<String>,
  pub removed: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: Option<bool>,
  pub nsfw: Option<bool>,
  pub actor_id: Option<DbUrl>,
  pub local: Option<bool>,
  pub private_key: Option<String>,
  pub public_key: Option<String>,
  pub last_refreshed_at: Option<chrono::NaiveDateTime>,
  pub icon: Option<Option<DbUrl>>,
  pub banner: Option<Option<DbUrl>>,
  pub followers_url: Option<DbUrl>,
  pub inbox_url: Option<DbUrl>,
  pub shared_inbox_url: Option<Option<DbUrl>>,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Community)]
#[table_name = "community_moderator"]
pub struct CommunityModerator {
  pub id: i32,
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "community_moderator"]
pub struct CommunityModeratorForm {
  pub community_id: CommunityId,
  pub person_id: PersonId,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Community)]
#[table_name = "community_person_ban"]
pub struct CommunityPersonBan {
  pub id: i32,
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "community_person_ban"]
pub struct CommunityPersonBanForm {
  pub community_id: CommunityId,
  pub person_id: PersonId,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Community)]
#[table_name = "community_follower"]
pub struct CommunityFollower {
  pub id: i32,
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub published: chrono::NaiveDateTime,
  pub pending: Option<bool>,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "community_follower"]
pub struct CommunityFollowerForm {
  pub community_id: CommunityId,
  pub person_id: PersonId,
  pub pending: bool,
}
