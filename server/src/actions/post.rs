extern crate diesel;
use schema::{post, post_like};
use diesel::*;
use diesel::result::Error;
use serde::{Deserialize, Serialize};
use {Crud, Likeable};

#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
#[table_name="post"]
pub struct Post {
  pub id: i32,
  pub name: String,
  pub url: Option<String>,
  pub body: Option<String>,
  pub attributed_to: String,
  pub community_id: i32,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name="post"]
pub struct PostForm {
  pub name: String,
  pub url: Option<String>,
  pub body: Option<String>,
  pub attributed_to: String,
  pub community_id: i32,
  pub updated: Option<chrono::NaiveDateTime>
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

#[derive(Insertable, AsChangeset, Clone)]
#[table_name="post_like"]
pub struct PostLikeForm {
  pub post_id: i32,
  pub fedi_user_id: String,
  pub score: i16
}

impl Crud<PostForm> for Post {
  fn read(conn: &PgConnection, post_id: i32) -> Result<Self, Error> {
    use schema::post::dsl::*;
    post.find(post_id)
      .first::<Self>(conn)
  }

  fn delete(conn: &PgConnection, post_id: i32) -> Result<usize, Error> {
    use schema::post::dsl::*;
    diesel::delete(post.find(post_id))
      .execute(conn)
  }

  fn create(conn: &PgConnection, new_post: &PostForm) -> Result<Self, Error> {
    use schema::post::dsl::*;
      insert_into(post)
        .values(new_post)
        .get_result::<Self>(conn)
  }

  fn update(conn: &PgConnection, post_id: i32, new_post: &PostForm) -> Result<Self, Error> {
    use schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set(new_post)
      .get_result::<Self>(conn)
  }
}

impl Likeable <PostLikeForm> for PostLike {
  fn like(conn: &PgConnection, post_like_form: &PostLikeForm) -> Result<Self, Error> {
    use schema::post_like::dsl::*;
    insert_into(post_like)
      .values(post_like_form)
      .get_result::<Self>(conn)
  }
  fn remove(conn: &PgConnection, post_like_form: &PostLikeForm) -> Result<usize, Error> {
    use schema::post_like::dsl::*;
    diesel::delete(post_like
      .filter(post_id.eq(post_like_form.post_id))
      .filter(fedi_user_id.eq(&post_like_form.fedi_user_id)))
      .execute(conn)
  }
}

#[cfg(test)]
mod tests {
  use establish_connection;
  use super::*;
  use Crud;
  use actions::community::*;
 #[test]
  fn test_crud() {
    let conn = establish_connection();

    let new_community = CommunityForm {
      name: "test community_2".to_string(),
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

    let expected_post = Post {
      id: inserted_post.id,
      name: "A test post".into(),
      url: None,
      body: None,
      attributed_to: "test_user.com".into(),
      community_id: inserted_community.id,
      published: inserted_post.published,
      updated: None
    };

    let post_like_form = PostLikeForm {
      post_id: inserted_post.id,
      fedi_user_id: "test".into(),
      score: 1
    };

    let inserted_post_like = PostLike::like(&conn, &post_like_form).unwrap();

    let expected_post_like = PostLike {
      id: inserted_post_like.id,
      post_id: inserted_post.id,
      fedi_user_id: "test".into(),
      published: inserted_post_like.published,
      score: 1
    };
    
    let read_post = Post::read(&conn, inserted_post.id).unwrap();
    let updated_post = Post::update(&conn, inserted_post.id, &new_post).unwrap();
    let like_removed = PostLike::remove(&conn, &post_like_form).unwrap();
    let num_deleted = Post::delete(&conn, inserted_post.id).unwrap();
    Community::delete(&conn, inserted_community.id).unwrap();

    assert_eq!(expected_post, read_post);
    assert_eq!(expected_post, inserted_post);
    assert_eq!(expected_post, updated_post);
    assert_eq!(expected_post_like, inserted_post_like);
    assert_eq!(1, like_removed);
    assert_eq!(1, num_deleted);

  }
}
