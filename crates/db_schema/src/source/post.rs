use crate::{
  naive_now,
  schema::{post, post_like, post_read, post_saved},
  CommunityId,
  DbUrl,
  PersonId,
  PostId,
};
use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use lemmy_apub_lib::traits::ApubObject;
use lemmy_utils::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
#[table_name = "post"]
pub struct Post {
  pub id: PostId,
  pub name: String,
  pub url: Option<DbUrl>,
  pub body: Option<String>,
  pub creator_id: PersonId,
  pub community_id: CommunityId,
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
  pub thumbnail_url: Option<DbUrl>,
  pub ap_id: DbUrl,
  pub local: bool,
}

#[derive(Insertable, AsChangeset, Default)]
#[table_name = "post"]
pub struct PostForm {
  pub name: String,
  pub creator_id: PersonId,
  pub community_id: CommunityId,
  pub nsfw: Option<bool>,
  pub url: Option<DbUrl>,
  pub body: Option<String>,
  pub removed: Option<bool>,
  pub locked: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: Option<bool>,
  pub stickied: Option<bool>,
  pub embed_title: Option<String>,
  pub embed_description: Option<String>,
  pub embed_html: Option<String>,
  pub thumbnail_url: Option<DbUrl>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Post)]
#[table_name = "post_like"]
pub struct PostLike {
  pub id: i32,
  pub post_id: PostId,
  pub person_id: PersonId,
  pub score: i16,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "post_like"]
pub struct PostLikeForm {
  pub post_id: PostId,
  pub person_id: PersonId,
  pub score: i16,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Post)]
#[table_name = "post_saved"]
pub struct PostSaved {
  pub id: i32,
  pub post_id: PostId,
  pub person_id: PersonId,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "post_saved"]
pub struct PostSavedForm {
  pub post_id: PostId,
  pub person_id: PersonId,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Post)]
#[table_name = "post_read"]
pub struct PostRead {
  pub id: i32,
  pub post_id: PostId,
  pub person_id: PersonId,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "post_read"]
pub struct PostReadForm {
  pub post_id: PostId,
  pub person_id: PersonId,
}

impl ApubObject for Post {
  type DataType = PgConnection;

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    None
  }

  fn read_from_apub_id(conn: &PgConnection, object_id: Url) -> Result<Option<Self>, LemmyError> {
    use crate::schema::post::dsl::*;
    let object_id: DbUrl = object_id.into();
    Ok(post.filter(ap_id.eq(object_id)).first::<Self>(conn).ok())
  }

  fn delete(self, conn: &PgConnection) -> Result<(), LemmyError> {
    use crate::schema::post::dsl::*;
    diesel::update(post.find(self.id))
      .set((deleted.eq(true), updated.eq(naive_now())))
      .get_result::<Self>(conn)?;
    Ok(())
  }
}
