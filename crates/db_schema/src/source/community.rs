use crate::{
  schema::{community, community_follower, community_moderator, community_user_ban},
  Url,
};
use serde::Serialize;

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "community"]
pub struct Community {
  pub id: i32,
  pub name: String,
  pub title: String,
  pub description: Option<String>,
  pub category_id: i32,
  pub creator_id: i32,
  pub removed: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub nsfw: bool,
  pub actor_id: Url,
  pub local: bool,
  pub private_key: Option<String>,
  pub public_key: Option<String>,
  pub last_refreshed_at: chrono::NaiveDateTime,
  pub icon: Option<String>,
  pub banner: Option<String>,
}

/// A safe representation of community, without the sensitive info
#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "community"]
pub struct CommunitySafe {
  pub id: i32,
  pub name: String,
  pub title: String,
  pub description: Option<String>,
  pub category_id: i32,
  pub creator_id: i32,
  pub removed: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub nsfw: bool,
  pub actor_id: Url,
  pub local: bool,
  pub icon: Option<String>,
  pub banner: Option<String>,
}

#[derive(Insertable, AsChangeset, Debug)]
#[table_name = "community"]
pub struct CommunityForm {
  pub name: String,
  pub title: String,
  pub description: Option<String>,
  pub category_id: i32,
  pub creator_id: i32,
  pub removed: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: Option<bool>,
  pub nsfw: bool,
  pub actor_id: Option<Url>,
  pub local: bool,
  pub private_key: Option<String>,
  pub public_key: Option<String>,
  pub last_refreshed_at: Option<chrono::NaiveDateTime>,
  pub icon: Option<Option<String>>,
  pub banner: Option<Option<String>>,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Community)]
#[table_name = "community_moderator"]
pub struct CommunityModerator {
  pub id: i32,
  pub community_id: i32,
  pub user_id: i32,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "community_moderator"]
pub struct CommunityModeratorForm {
  pub community_id: i32,
  pub user_id: i32,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Community)]
#[table_name = "community_user_ban"]
pub struct CommunityUserBan {
  pub id: i32,
  pub community_id: i32,
  pub user_id: i32,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "community_user_ban"]
pub struct CommunityUserBanForm {
  pub community_id: i32,
  pub user_id: i32,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Community)]
#[table_name = "community_follower"]
pub struct CommunityFollower {
  pub id: i32,
  pub community_id: i32,
  pub user_id: i32,
  pub published: chrono::NaiveDateTime,
  pub pending: Option<bool>,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "community_follower"]
pub struct CommunityFollowerForm {
  pub community_id: i32,
  pub user_id: i32,
  pub pending: bool,
}
