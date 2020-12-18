use crate::{
  naive_now,
  schema::{post, post_like, post_read, post_saved},
};
use diesel::{result::Error, *};
use serde::Serialize;
use url::{ParseError, Url};

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
  pub ap_id: String,
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
  pub ap_id: Option<String>,
  pub local: bool,
}

impl Post {
  pub fn read(conn: &PgConnection, post_id: i32) -> Result<Self, Error> {
    use crate::schema::post::dsl::*;
    post.filter(id.eq(post_id)).first::<Self>(conn)
  }

  pub fn list_for_community(
    conn: &PgConnection,
    the_community_id: i32,
  ) -> Result<Vec<Self>, Error> {
    use crate::schema::post::dsl::*;
    post
      .filter(community_id.eq(the_community_id))
      .then_order_by(published.desc())
      .then_order_by(stickied.desc())
      .limit(20)
      .load::<Self>(conn)
  }

  pub fn update_ap_id(conn: &PgConnection, post_id: i32, apub_id: String) -> Result<Self, Error> {
    use crate::schema::post::dsl::*;

    diesel::update(post.find(post_id))
      .set(ap_id.eq(apub_id))
      .get_result::<Self>(conn)
  }

  pub fn permadelete_for_creator(
    conn: &PgConnection,
    for_creator_id: i32,
  ) -> Result<Vec<Self>, Error> {
    use crate::schema::post::dsl::*;

    let perma_deleted = "*Permananently Deleted*";
    let perma_deleted_url = "https://deleted.com";

    diesel::update(post.filter(creator_id.eq(for_creator_id)))
      .set((
        name.eq(perma_deleted),
        url.eq(perma_deleted_url),
        body.eq(perma_deleted),
        deleted.eq(true),
        updated.eq(naive_now()),
      ))
      .get_results::<Self>(conn)
  }

  pub fn update_deleted(
    conn: &PgConnection,
    post_id: i32,
    new_deleted: bool,
  ) -> Result<Self, Error> {
    use crate::schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set((deleted.eq(new_deleted), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  pub fn update_removed(
    conn: &PgConnection,
    post_id: i32,
    new_removed: bool,
  ) -> Result<Self, Error> {
    use crate::schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set((removed.eq(new_removed), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  pub fn update_removed_for_creator(
    conn: &PgConnection,
    for_creator_id: i32,
    for_community_id: Option<i32>,
    new_removed: bool,
  ) -> Result<Vec<Self>, Error> {
    use crate::schema::post::dsl::*;

    let mut update = diesel::update(post).into_boxed();
    update = update.filter(creator_id.eq(for_creator_id));

    if let Some(for_community_id) = for_community_id {
      update = update.filter(community_id.eq(for_community_id));
    }

    update
      .set((removed.eq(new_removed), updated.eq(naive_now())))
      .get_results::<Self>(conn)
  }

  pub fn update_locked(conn: &PgConnection, post_id: i32, new_locked: bool) -> Result<Self, Error> {
    use crate::schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set(locked.eq(new_locked))
      .get_result::<Self>(conn)
  }

  pub fn update_stickied(
    conn: &PgConnection,
    post_id: i32,
    new_stickied: bool,
  ) -> Result<Self, Error> {
    use crate::schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set(stickied.eq(new_stickied))
      .get_result::<Self>(conn)
  }

  pub fn is_post_creator(user_id: i32, post_creator_id: i32) -> bool {
    user_id == post_creator_id
  }
}

impl PostForm {
  pub fn get_ap_id(&self) -> Result<Url, ParseError> {
    Url::parse(&self.ap_id.as_ref().unwrap_or(&"not_a_url".to_string()))
  }
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
