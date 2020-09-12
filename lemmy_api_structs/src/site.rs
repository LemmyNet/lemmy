use lemmy_db::{
  category::*,
  comment_view::*,
  community_view::*,
  moderator_views::*,
  post_view::*,
  site_view::*,
  user::*,
  user_view::*,
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
  pub users: Vec<UserView>,
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
  pub site: SiteView,
}

#[derive(Serialize)]
pub struct GetSiteResponse {
  pub site: Option<SiteView>,
  pub admins: Vec<UserView>,
  pub banned: Vec<UserView>,
  pub online: usize,
  pub version: String,
  pub my_user: Option<User_>,
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
