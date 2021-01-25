use crate::{
  schema::{post, post_like, post_read, post_saved},
  Url,
};
use serde::Serialize;

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "post"]
pub struct Post {
  pub id: i32,
  pub name: String,
  pub url: Option<String>,
  pub body: Option<String>,
  pub creator_id: i32,
  pub community_id: i32,
  pub removed: bool,
  pub locked: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub nsfw: bool,
  pub stickied: bool,
  pub embed_title: Option<String>,
  pub embed_description: Option<String>,
  pub embed_html: Option<String>,
  pub thumbnail_url: Option<String>,
  pub ap_id: Url,
  pub local: bool,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "post"]
pub struct PostForm {
  pub name: String,
  pub url: Option<String>,
  pub body: Option<String>,
  pub creator_id: i32,
  pub community_id: i32,
  pub removed: Option<bool>,
  pub locked: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: Option<bool>,
  pub nsfw: bool,
  pub stickied: Option<bool>,
  pub embed_title: Option<String>,
  pub embed_description: Option<String>,
  pub embed_html: Option<String>,
  pub thumbnail_url: Option<String>,
  pub ap_id: Option<Url>,
  pub local: bool,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Post)]
#[table_name = "post_like"]
pub struct PostLike {
  pub id: i32,
  pub post_id: i32,
  pub user_id: i32,
  pub score: i16,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "post_like"]
pub struct PostLikeForm {
  pub post_id: i32,
  pub user_id: i32,
  pub score: i16,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Post)]
#[table_name = "post_saved"]
pub struct PostSaved {
  pub id: i32,
  pub post_id: i32,
  pub user_id: i32,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "post_saved"]
pub struct PostSavedForm {
  pub post_id: i32,
  pub user_id: i32,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Post)]
#[table_name = "post_read"]
pub struct PostRead {
  pub id: i32,

  pub post_id: i32,

  pub user_id: i32,

  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "post_read"]
pub struct PostReadForm {
  pub post_id: i32,

  pub user_id: i32,
}
