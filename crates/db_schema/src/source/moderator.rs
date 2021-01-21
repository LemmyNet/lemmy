use crate::schema::{
  mod_add,
  mod_add_community,
  mod_ban,
  mod_ban_from_community,
  mod_lock_post,
  mod_remove_comment,
  mod_remove_community,
  mod_remove_post,
  mod_sticky_post,
};
use serde::Serialize;

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "mod_remove_post"]
pub struct ModRemovePost {
  pub id: i32,
  pub mod_user_id: i32,
  pub post_id: i32,
  pub reason: Option<String>,
  pub removed: Option<bool>,
  pub when_: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "mod_remove_post"]
pub struct ModRemovePostForm {
  pub mod_user_id: i32,
  pub post_id: i32,
  pub reason: Option<String>,
  pub removed: Option<bool>,
}

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "mod_lock_post"]
pub struct ModLockPost {
  pub id: i32,
  pub mod_user_id: i32,
  pub post_id: i32,
  pub locked: Option<bool>,
  pub when_: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "mod_lock_post"]
pub struct ModLockPostForm {
  pub mod_user_id: i32,
  pub post_id: i32,
  pub locked: Option<bool>,
}

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "mod_sticky_post"]
pub struct ModStickyPost {
  pub id: i32,
  pub mod_user_id: i32,
  pub post_id: i32,
  pub stickied: Option<bool>,
  pub when_: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "mod_sticky_post"]
pub struct ModStickyPostForm {
  pub mod_user_id: i32,
  pub post_id: i32,
  pub stickied: Option<bool>,
}

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "mod_remove_comment"]
pub struct ModRemoveComment {
  pub id: i32,
  pub mod_user_id: i32,
  pub comment_id: i32,
  pub reason: Option<String>,
  pub removed: Option<bool>,
  pub when_: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "mod_remove_comment"]
pub struct ModRemoveCommentForm {
  pub mod_user_id: i32,
  pub comment_id: i32,
  pub reason: Option<String>,
  pub removed: Option<bool>,
}

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "mod_remove_community"]
pub struct ModRemoveCommunity {
  pub id: i32,
  pub mod_user_id: i32,
  pub community_id: i32,
  pub reason: Option<String>,
  pub removed: Option<bool>,
  pub expires: Option<chrono::NaiveDateTime>,
  pub when_: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "mod_remove_community"]
pub struct ModRemoveCommunityForm {
  pub mod_user_id: i32,
  pub community_id: i32,
  pub reason: Option<String>,
  pub removed: Option<bool>,
  pub expires: Option<chrono::NaiveDateTime>,
}

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "mod_ban_from_community"]
pub struct ModBanFromCommunity {
  pub id: i32,
  pub mod_user_id: i32,
  pub other_user_id: i32,
  pub community_id: i32,
  pub reason: Option<String>,
  pub banned: Option<bool>,
  pub expires: Option<chrono::NaiveDateTime>,
  pub when_: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "mod_ban_from_community"]
pub struct ModBanFromCommunityForm {
  pub mod_user_id: i32,
  pub other_user_id: i32,
  pub community_id: i32,
  pub reason: Option<String>,
  pub banned: Option<bool>,
  pub expires: Option<chrono::NaiveDateTime>,
}

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "mod_ban"]
pub struct ModBan {
  pub id: i32,
  pub mod_user_id: i32,
  pub other_user_id: i32,
  pub reason: Option<String>,
  pub banned: Option<bool>,
  pub expires: Option<chrono::NaiveDateTime>,
  pub when_: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "mod_ban"]
pub struct ModBanForm {
  pub mod_user_id: i32,
  pub other_user_id: i32,
  pub reason: Option<String>,
  pub banned: Option<bool>,
  pub expires: Option<chrono::NaiveDateTime>,
}

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "mod_add_community"]
pub struct ModAddCommunity {
  pub id: i32,
  pub mod_user_id: i32,
  pub other_user_id: i32,
  pub community_id: i32,
  pub removed: Option<bool>,
  pub when_: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "mod_add_community"]
pub struct ModAddCommunityForm {
  pub mod_user_id: i32,
  pub other_user_id: i32,
  pub community_id: i32,
  pub removed: Option<bool>,
}

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "mod_add"]
pub struct ModAdd {
  pub id: i32,
  pub mod_user_id: i32,
  pub other_user_id: i32,
  pub removed: Option<bool>,
  pub when_: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "mod_add"]
pub struct ModAddForm {
  pub mod_user_id: i32,
  pub other_user_id: i32,
  pub removed: Option<bool>,
}
