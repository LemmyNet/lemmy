extern crate diesel;
use schema::{comment, comment_like};
use diesel::*;
use diesel::result::Error;
use {Crud, Likeable};

// WITH RECURSIVE MyTree AS (
//     SELECT * FROM comment WHERE parent_id IS NULL
//     UNION ALL
//     SELECT m.* FROM comment AS m JOIN MyTree AS t ON m.parent_id = t.id
// )
// SELECT * FROM MyTree;

#[derive(Queryable, Identifiable, PartialEq, Debug)]
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

#[derive(Insertable, AsChangeset, Clone, Copy)]
#[table_name="comment"]
pub struct CommentForm<'a> {
  pub content: &'a str,
  pub attributed_to: &'a str,
  pub post_id: &'a i32,
  pub parent_id: Option<&'a i32>,
  pub updated: Option<&'a chrono::NaiveDateTime>
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

#[derive(Insertable, AsChangeset, Clone, Copy)]
#[table_name="comment_like"]
pub struct CommentLikeForm<'a> {
  pub comment_id: &'a i32,
  pub fedi_user_id: &'a str,
  pub score: &'a i16
}

impl<'a> Crud<CommentForm<'a>> for Comment {
  fn read(conn: &PgConnection, comment_id: i32) -> Comment {
    use schema::comment::dsl::*;
    comment.find(comment_id)
      .first::<Comment>(conn)
      .expect("Error in query")
  }

  fn delete(conn: &PgConnection, comment_id: i32) -> usize {
    use schema::comment::dsl::*;
    diesel::delete(comment.find(comment_id))
      .execute(conn)
      .expect("Error deleting.")
  }

  fn create(conn: &PgConnection, comment_form: CommentForm) -> Result<Comment, Error> {
    use schema::comment::dsl::*;
      insert_into(comment)
        .values(comment_form)
        .get_result::<Comment>(conn)
  }

  fn update(conn: &PgConnection, comment_id: i32, comment_form: CommentForm) -> Comment {
    use schema::comment::dsl::*;
    diesel::update(comment.find(comment_id))
      .set(comment_form)
      .get_result::<Comment>(conn)
      .expect(&format!("Unable to find {}", comment_id))
  }
}

impl<'a> Likeable <CommentLikeForm<'a>> for CommentLike {
  fn like(conn: &PgConnection, comment_like_form: CommentLikeForm) -> Result<CommentLike, Error> {
    use schema::comment_like::dsl::*;
    insert_into(comment_like)
      .values(comment_like_form)
      .get_result::<CommentLike>(conn)
  }
  fn remove(conn: &PgConnection, comment_like_form: CommentLikeForm) -> usize {
    use schema::comment_like::dsl::*;
    diesel::delete(comment_like
      .filter(comment_id.eq(comment_like_form.comment_id))
      .filter(fedi_user_id.eq(comment_like_form.fedi_user_id)))
      .execute(conn)
      .expect("Error deleting.")
  }
}

#[cfg(test)]
mod tests {
  use establish_connection;
  use super::*;
  use actions::post::*;
  use Crud;
 #[test]
  fn test_crud() {
    let conn = establish_connection();
    
    let new_post = PostForm {
      name: "A test post".into(),
      url: "https://test.com".into(),
      attributed_to: "test_user.com".into(),
      updated: None
    };

    let inserted_post = Post::create(&conn, new_post).unwrap();

    let comment_form = CommentForm {
      content: "A test comment".into(),
      attributed_to: "test_user.com".into(),
      post_id: &inserted_post.id,
      parent_id: None,
      updated: None
    };

    let inserted_comment = Comment::create(&conn, comment_form).unwrap();

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
      post_id: &inserted_post.id,
      parent_id: Some(&inserted_comment.id),
      updated: None
    };

    let inserted_child_comment = Comment::create(&conn, child_comment_form).unwrap();

    let comment_like_form = CommentLikeForm {
      comment_id: &inserted_comment.id,
      fedi_user_id: "test".into(),
      score: &1
    };

    let inserted_comment_like = CommentLike::like(&conn, comment_like_form).unwrap();

    let expected_comment_like = CommentLike {
      id: inserted_comment_like.id,
      comment_id: inserted_comment.id,
      fedi_user_id: "test".into(),
      published: inserted_comment_like.published,
      score: 1
    };
    
    let read_comment = Comment::read(&conn, inserted_comment.id);
    let updated_comment = Comment::update(&conn, inserted_comment.id, comment_form);
    let like_removed = CommentLike::remove(&conn, comment_like_form);
    let num_deleted = Comment::delete(&conn, inserted_comment.id);
    Comment::delete(&conn, inserted_child_comment.id);
    Post::delete(&conn, inserted_post.id);

    assert_eq!(expected_comment, read_comment);
    assert_eq!(expected_comment, inserted_comment);
    assert_eq!(expected_comment, updated_comment);
    assert_eq!(expected_comment_like, inserted_comment_like);
    assert_eq!(expected_comment.id, inserted_child_comment.parent_id.unwrap());
    assert_eq!(1, like_removed);
    assert_eq!(1, num_deleted);

  }
}
