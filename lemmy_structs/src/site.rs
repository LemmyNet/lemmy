use lemmy_db_schema::source::{category::*, user::UserSafeSettings};
use lemmy_db_views::{comment_view::CommentView, post_view::PostView, site_view::SiteView};
use lemmy_db_views_actor::{community_view::CommunityView, user_view::UserViewSafe};
use lemmy_db_views_moderator::{
  mod_add_community_view::ModAddCommunityView,
  mod_add_view::ModAddView,
  mod_ban_from_community_view::ModBanFromCommunityView,
  mod_ban_view::ModBanView,
  mod_lock_post_view::ModLockPostView,
  mod_remove_comment_view::ModRemoveCommentView,
  mod_remove_community_view::ModRemoveCommunityView,
  mod_remove_post_view::ModRemovePostView,
  mod_sticky_post_view::ModStickyPostView,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ListCategories {}

#[derive(Serialize)]
pub struct ListCategoriesResponse {
  pub categories: Vec<Category>,
}

#[derive(Deserialize, Debug)]
pub struct Search {
  pub q: String,
  pub type_: String,
  pub community_id: Option<i32>,
  pub community_name: Option<String>,
  pub sort: String,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct SearchResponse {
  pub type_: String,
  pub comments: Vec<CommentView>,
  pub posts: Vec<PostView>,
  pub communities: Vec<CommunityView>,
  pub users: Vec<UserViewSafe>,
}

#[derive(Deserialize)]
pub struct GetModlog {
  pub mod_user_id: Option<i32>,
  pub community_id: Option<i32>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct GetModlogResponse {
  pub removed_posts: Vec<ModRemovePostView>,
  pub locked_posts: Vec<ModLockPostView>,
  pub stickied_posts: Vec<ModStickyPostView>,
  pub removed_comments: Vec<ModRemoveCommentView>,
  pub removed_communities: Vec<ModRemoveCommunityView>,
  pub banned_from_community: Vec<ModBanFromCommunityView>,
  pub banned: Vec<ModBanView>,
  pub added_to_community: Vec<ModAddCommunityView>,
  pub added: Vec<ModAddView>,
}

#[derive(Deserialize)]
pub struct CreateSite {
  pub name: String,
  pub description: Option<String>,
  pub icon: Option<String>,
  pub banner: Option<String>,
  pub enable_downvotes: bool,
  pub open_registration: bool,
  pub enable_nsfw: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct EditSite {
  pub name: String,
  pub description: Option<String>,
  pub icon: Option<String>,
  pub banner: Option<String>,
  pub enable_downvotes: bool,
  pub open_registration: bool,
  pub enable_nsfw: bool,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct GetSite {
  pub auth: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct SiteResponse {
  pub site_view: SiteView,
}

#[derive(Serialize)]
pub struct GetSiteResponse {
  pub site_view: Option<SiteView>, // Because the site might not be set up yet
  pub admins: Vec<UserViewSafe>,
  pub banned: Vec<UserViewSafe>,
  pub online: usize,
  pub version: String,
  pub my_user: Option<UserSafeSettings>,
  pub federated_instances: Vec<String>,
}

#[derive(Deserialize)]
pub struct TransferSite {
  pub user_id: i32,
  pub auth: String,
}

#[derive(Deserialize)]
pub struct GetSiteConfig {
  pub auth: String,
}

#[derive(Serialize)]
pub struct GetSiteConfigResponse {
  pub config_hjson: String,
}

#[derive(Deserialize)]
pub struct SaveSiteConfig {
  pub config_hjson: String,
  pub auth: String,
}
