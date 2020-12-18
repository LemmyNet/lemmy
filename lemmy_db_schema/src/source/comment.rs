use crate::{
  naive_now,
  schema::{comment, comment_alias_1, comment_like, comment_saved},
  source::post::Post,
};
use diesel::{result::Error, PgConnection, *};
use serde::Serialize;
use url::{ParseError, Url};

// WITH RECURSIVE MyTree AS (
//     SELECT * FROM comment WHERE parent_id IS NULL
//     UNION ALL
//     SELECT m.* FROM comment AS m JOIN MyTree AS t ON m.parent_id = t.id
// )
// SELECT * FROM MyTree;

#[derive(Clone, Queryable, Associations, Identifiable, PartialEq, Debug, Serialize)]
#[belongs_to(Post)]
#[table_name = "comment"]
pub struct Comment {
  pub id: i32,
  pub creator_id: i32,
  pub post_id: i32,
  pub parent_id: Option<i32>,
  pub content: String,
  pub removed: bool,
  pub read: bool, // Whether the recipient has read the comment or not
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub ap_id: String,
  pub local: bool,
}

#[derive(Clone, Queryable, Associations, Identifiable, PartialEq, Debug, Serialize)]
#[belongs_to(Post)]
#[table_name = "comment_alias_1"]
pub struct CommentAlias1 {
  pub id: i32,
  pub creator_id: i32,
  pub post_id: i32,
  pub parent_id: Option<i32>,
  pub content: String,
  pub removed: bool,
  pub read: bool, // Whether the recipient has read the comment or not
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub ap_id: String,
  pub local: bool,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "comment"]
pub struct CommentForm {
  pub creator_id: i32,
  pub post_id: i32,
  pub parent_id: Option<i32>,
  pub content: String,
  pub removed: Option<bool>,
  pub read: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: Option<bool>,
  pub ap_id: Option<String>,
  pub local: bool,
}

impl Comment {
  pub fn update_ap_id(
    conn: &PgConnection,
    comment_id: i32,
    apub_id: String,
  ) -> Result<Self, Error> {
    use crate::schema::comment::dsl::*;

    diesel::update(comment.find(comment_id))
      .set(ap_id.eq(apub_id))
      .get_result::<Self>(conn)
  }

  pub fn permadelete_for_creator(
    conn: &PgConnection,
    for_creator_id: i32,
  ) -> Result<Vec<Self>, Error> {
    use crate::schema::comment::dsl::*;
    diesel::update(comment.filter(creator_id.eq(for_creator_id)))
      .set((
        content.eq("*Permananently Deleted*"),
        deleted.eq(true),
        updated.eq(naive_now()),
      ))
      .get_results::<Self>(conn)
  }

  pub fn update_deleted(
    conn: &PgConnection,
    comment_id: i32,
    new_deleted: bool,
  ) -> Result<Self, Error> {
    use crate::schema::comment::dsl::*;
    diesel::update(comment.find(comment_id))
      .set((deleted.eq(new_deleted), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  pub fn update_removed(
    conn: &PgConnection,
    comment_id: i32,
    new_removed: bool,
  ) -> Result<Self, Error> {
    use crate::schema::comment::dsl::*;
    diesel::update(comment.find(comment_id))
      .set((removed.eq(new_removed), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  pub fn update_removed_for_creator(
    conn: &PgConnection,
    for_creator_id: i32,
    new_removed: bool,
  ) -> Result<Vec<Self>, Error> {
    use crate::schema::comment::dsl::*;
    diesel::update(comment.filter(creator_id.eq(for_creator_id)))
      .set((removed.eq(new_removed), updated.eq(naive_now())))
      .get_results::<Self>(conn)
  }

  pub fn update_read(conn: &PgConnection, comment_id: i32, new_read: bool) -> Result<Self, Error> {
    use crate::schema::comment::dsl::*;
    diesel::update(comment.find(comment_id))
      .set(read.eq(new_read))
      .get_result::<Self>(conn)
  }

  pub fn update_content(
    conn: &PgConnection,
    comment_id: i32,
    new_content: &str,
  ) -> Result<Self, Error> {
    use crate::schema::comment::dsl::*;
    diesel::update(comment.find(comment_id))
      .set((content.eq(new_content), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }
}

impl CommentForm {
  pub fn get_ap_id(&self) -> Result<Url, ParseError> {
    Url::parse(&self.ap_id.as_ref().unwrap_or(&"not_a_url".to_string()))
  }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug, Clone)]
#[belongs_to(Comment)]
#[table_name = "comment_like"]
pub struct CommentLike {
  pub id: i32,
  pub user_id: i32,
  pub comment_id: i32,
  pub post_id: i32, // TODO this is redundant
  pub score: i16,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "comment_like"]
pub struct CommentLikeForm {
  pub user_id: i32,
  pub comment_id: i32,
  pub post_id: i32, // TODO this is redundant
  pub score: i16,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Comment)]
#[table_name = "comment_saved"]
pub struct CommentSaved {
  pub id: i32,
  pub comment_id: i32,
  pub user_id: i32,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "comment_saved"]
pub struct CommentSavedForm {
  pub comment_id: i32,
  pub user_id: i32,
}
