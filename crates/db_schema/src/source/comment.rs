use crate::{
  schema::{comment, comment_alias_1, comment_like, comment_saved},
  source::post::Post,
  CommentId,
  DbUrl,
  PersonId,
  PostId,
};
use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use lemmy_apub_lib::traits::ApubObject;
use lemmy_utils::LemmyError;
use serde::Serialize;
use url::Url;

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
  pub id: CommentId,
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub parent_id: Option<CommentId>,
  pub content: String,
  pub removed: bool,
  pub read: bool, // Whether the recipient has read the comment or not
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub ap_id: DbUrl,
  pub local: bool,
}

#[derive(Clone, Queryable, Associations, Identifiable, PartialEq, Debug, Serialize)]
#[belongs_to(Post)]
#[table_name = "comment_alias_1"]
pub struct CommentAlias1 {
  pub id: CommentId,
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub parent_id: Option<CommentId>,
  pub content: String,
  pub removed: bool,
  pub read: bool, // Whether the recipient has read the comment or not
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub ap_id: DbUrl,
  pub local: bool,
}

#[derive(Insertable, AsChangeset, Clone, Default)]
#[table_name = "comment"]
pub struct CommentForm {
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub content: String,
  pub parent_id: Option<CommentId>,
  pub removed: Option<bool>,
  pub read: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: Option<bool>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug, Clone)]
#[belongs_to(Comment)]
#[table_name = "comment_like"]
pub struct CommentLike {
  pub id: i32,
  pub person_id: PersonId,
  pub comment_id: CommentId,
  pub post_id: PostId, // TODO this is redundant
  pub score: i16,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "comment_like"]
pub struct CommentLikeForm {
  pub person_id: PersonId,
  pub comment_id: CommentId,
  pub post_id: PostId, // TODO this is redundant
  pub score: i16,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Comment)]
#[table_name = "comment_saved"]
pub struct CommentSaved {
  pub id: i32,
  pub comment_id: CommentId,
  pub person_id: PersonId,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "comment_saved"]
pub struct CommentSavedForm {
  pub comment_id: CommentId,
  pub person_id: PersonId,
}

impl ApubObject for Comment {
  type DataType = PgConnection;

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    None
  }

  fn read_from_apub_id(conn: &PgConnection, object_id: Url) -> Result<Option<Self>, LemmyError> {
    use crate::schema::comment::dsl::*;
    let object_id: DbUrl = object_id.into();
    Ok(comment.filter(ap_id.eq(object_id)).first::<Self>(conn).ok())
  }
}
