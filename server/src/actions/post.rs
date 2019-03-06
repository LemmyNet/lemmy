extern crate diesel;
use schema::{post, post_like};
use diesel::*;
use diesel::result::Error;
use {Crud, Likeable};

#[derive(Queryable, Identifiable, PartialEq, Debug)]
#[table_name="post"]
pub struct Post {
  pub id: i32,
  pub name: String,
  pub url: String,
  pub attributed_to: String,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>
}

#[derive(Insertable, AsChangeset, Clone, Copy)]
#[table_name="post"]
pub struct PostForm<'a> {
  pub name: &'a str,
  pub url: &'a str,
  pub attributed_to: &'a str,
  pub updated: Option<&'a chrono::NaiveDateTime>
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Post)]
#[table_name = "post_like"]
pub struct PostLike {
  pub id: i32,
  pub post_id: i32,
  pub fedi_user_id: String,
  pub score: i16,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone, Copy)]
#[table_name="post_like"]
pub struct PostLikeForm<'a> {
  pub post_id: &'a i32,
  pub fedi_user_id: &'a str,
  pub score: &'a i16
}

impl<'a> Crud<PostForm<'a>> for Post {
  fn read(conn: &PgConnection, post_id: i32) -> Post {
    use schema::post::dsl::*;
    post.find(post_id)
      .first::<Post>(conn)
      .expect("Error in query")
  }

  fn delete(conn: &PgConnection, post_id: i32) -> usize {
    use schema::post::dsl::*;
    diesel::delete(post.find(post_id))
      .execute(conn)
      .expect("Error deleting.")
  }

  fn create(conn: &PgConnection, new_post: PostForm) -> Result<Post, Error> {
    use schema::post::dsl::*;
      insert_into(post)
        .values(new_post)
        .get_result::<Post>(conn)
  }

  fn update(conn: &PgConnection, post_id: i32, new_post: PostForm) -> Post {
    use schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set(new_post)
      .get_result::<Post>(conn)
      .expect(&format!("Unable to find {}", post_id))
  }
}

impl<'a> Likeable <PostLikeForm<'a>> for PostLike {
  fn like(conn: &PgConnection, post_like_form: PostLikeForm) -> Result<PostLike, Error> {
    use schema::post_like::dsl::*;
    insert_into(post_like)
      .values(post_like_form)
      .get_result::<PostLike>(conn)
  }
  fn remove(conn: &PgConnection, post_like_form: PostLikeForm) -> usize {
    use schema::post_like::dsl::*;
    diesel::delete(post_like
      .filter(post_id.eq(post_like_form.post_id))
      .filter(fedi_user_id.eq(post_like_form.fedi_user_id)))
      .execute(conn)
      .expect("Error deleting.")
  }
}

#[cfg(test)]
mod tests {
  use establish_connection;
  use super::*;
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

    let expected_post = Post {
      id: inserted_post.id,
      name: "A test post".into(),
      url: "https://test.com".into(),
      attributed_to: "test_user.com".into(),
      published: inserted_post.published,
      updated: None
    };

    let post_like_form = PostLikeForm {
      post_id: &inserted_post.id,
      fedi_user_id: "test".into(),
      score: &1
    };

    let inserted_post_like = PostLike::like(&conn, post_like_form).unwrap();

    let expected_post_like = PostLike {
      id: inserted_post_like.id,
      post_id: inserted_post.id,
      fedi_user_id: "test".into(),
      published: inserted_post_like.published,
      score: 1
    };
    
    let read_post = Post::read(&conn, inserted_post.id);
    let updated_post = Post::update(&conn, inserted_post.id, new_post);
    let like_removed = PostLike::remove(&conn, post_like_form);
    let num_deleted = Post::delete(&conn, inserted_post.id);

    assert_eq!(expected_post, read_post);
    assert_eq!(expected_post, inserted_post);
    assert_eq!(expected_post, updated_post);
    assert_eq!(expected_post_like, inserted_post_like);
    assert_eq!(1, like_removed);
    assert_eq!(1, num_deleted);

  }
}
