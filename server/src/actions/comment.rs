extern crate diesel;
use schema::{comment, comment_like};
use diesel::*;
use diesel::result::Error;
use serde::{Deserialize, Serialize};
use {Crud, Likeable};
use actions::post::Post;

// WITH RECURSIVE MyTree AS (
//     SELECT * FROM comment WHERE parent_id IS NULL
//     UNION ALL
//     SELECT m.* FROM comment AS m JOIN MyTree AS t ON m.parent_id = t.id
// )
// SELECT * FROM MyTree;

#[derive(Queryable, Associations, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
#[belongs_to(Post)]
#[table_name="comment"]
pub struct Comment {
  pub id: i32,
  pub content: String,
  pub attributed_to: String,
  pub post_id: i32,
  pub parent_id: Option<i32>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name="comment"]
pub struct CommentForm {
  pub content: String,
  pub attributed_to: String,
  pub post_id: i32,
  pub parent_id: Option<i32>,
  pub updated: Option<chrono::NaiveDateTime>
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Comment)]
#[table_name = "comment_like"]
pub struct CommentLike {
  pub id: i32,
  pub comment_id: i32,
  pub fedi_user_id: String,
  pub score: i16,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name="comment_like"]
pub struct CommentLikeForm {
  pub comment_id: i32,
  pub fedi_user_id: String,
  pub score: i16
}

impl Crud<CommentForm> for Comment {
  fn read(conn: &PgConnection, comment_id: i32) -> Result<Self, Error> {
    use schema::comment::dsl::*;
    comment.find(comment_id)
      .first::<Self>(conn)
  }

  fn delete(conn: &PgConnection, comment_id: i32) -> Result<usize, Error> {
    use schema::comment::dsl::*;
    diesel::delete(comment.find(comment_id))
      .execute(conn)
  }

  fn create(conn: &PgConnection, comment_form: &CommentForm) -> Result<Self, Error> {
    use schema::comment::dsl::*;
      insert_into(comment)
        .values(comment_form)
        .get_result::<Self>(conn)
  }

  fn update(conn: &PgConnection, comment_id: i32, comment_form: &CommentForm) -> Result<Self, Error> {
    use schema::comment::dsl::*;
    diesel::update(comment.find(comment_id))
      .set(comment_form)
      .get_result::<Self>(conn)
  }
}

impl Likeable <CommentLikeForm> for CommentLike {
  fn like(conn: &PgConnection, comment_like_form: &CommentLikeForm) -> Result<Self, Error> {
    use schema::comment_like::dsl::*;
    insert_into(comment_like)
      .values(comment_like_form)
      .get_result::<Self>(conn)
  }
  fn remove(conn: &PgConnection, comment_like_form: &CommentLikeForm) -> Result<usize, Error> {
    use schema::comment_like::dsl::*;
    diesel::delete(comment_like
      .filter(comment_id.eq(comment_like_form.comment_id))
      .filter(fedi_user_id.eq(&comment_like_form.fedi_user_id)))
      .execute(conn)
  }
}

impl Comment {
  pub fn from_post(conn: &PgConnection, post: &Post) -> Result<Vec<Self>, Error> {
    use schema::community::dsl::*;
    Comment::belonging_to(post)
      .load::<Self>(conn) 
  }
}

#[cfg(test)]
mod tests {
  use establish_connection;
  use super::*;
  use actions::post::*;
  use actions::community::*;
  use Crud;
 #[test]
  fn test_crud() {
    let conn = establish_connection();

    let new_community = CommunityForm {
      name: "test community".to_string(),
      updated: None
    };

    let inserted_community = Community::create(&conn, &new_community).unwrap();
    
    let new_post = PostForm {
      name: "A test post".into(),
      url: None,
      body: None,
      attributed_to: "test_user.com".into(),
      community_id: inserted_community.id,
      updated: None
    };

    let inserted_post = Post::create(&conn, &new_post).unwrap();

    let comment_form = CommentForm {
      content: "A test comment".into(),
      attributed_to: "test_user.com".into(),
      post_id: inserted_post.id,
      parent_id: None,
      updated: None
    };

    let inserted_comment = Comment::create(&conn, &comment_form).unwrap();

    let expected_comment = Comment {
      id: inserted_comment.id,
      content: "A test comment".into(),
      attributed_to: "test_user.com".into(),
      post_id: inserted_post.id,
      parent_id: None,
      published: inserted_comment.published,
      updated: None
    };
    
    let child_comment_form = CommentForm {
      content: "A child comment".into(),
      attributed_to: "test_user.com".into(),
      post_id: inserted_post.id,
      parent_id: Some(inserted_comment.id),
      updated: None
    };

    let inserted_child_comment = Comment::create(&conn, &child_comment_form).unwrap();

    let comment_like_form = CommentLikeForm {
      comment_id: inserted_comment.id,
      fedi_user_id: "test".into(),
      score: 1
    };

    let inserted_comment_like = CommentLike::like(&conn, &comment_like_form).unwrap();

    let expected_comment_like = CommentLike {
      id: inserted_comment_like.id,
      comment_id: inserted_comment.id,
      fedi_user_id: "test".into(),
      published: inserted_comment_like.published,
      score: 1
    };
    
    let read_comment = Comment::read(&conn, inserted_comment.id).unwrap();
    let updated_comment = Comment::update(&conn, inserted_comment.id, &comment_form).unwrap();
    let like_removed = CommentLike::remove(&conn, &comment_like_form).unwrap();
    let num_deleted = Comment::delete(&conn, inserted_comment.id).unwrap();
    Comment::delete(&conn, inserted_child_comment.id).unwrap();
    Post::delete(&conn, inserted_post.id).unwrap();
    Community::delete(&conn, inserted_community.id).unwrap();

    assert_eq!(expected_comment, read_comment);
    assert_eq!(expected_comment, inserted_comment);
    assert_eq!(expected_comment, updated_comment);
    assert_eq!(expected_comment_like, inserted_comment_like);
    assert_eq!(expected_comment.id, inserted_child_comment.parent_id.unwrap());
    assert_eq!(1, like_removed);
    assert_eq!(1, num_deleted);

  }
}
