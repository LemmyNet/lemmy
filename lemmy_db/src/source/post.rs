use crate::{ApubObject, Crud, Likeable, Readable, Saveable};
use diesel::{dsl::*, result::Error, *};
use lemmy_db_schema::source::post::{
  Post,
  PostForm,
  PostLike,
  PostLikeForm,
  PostRead,
  PostReadForm,
  PostSaved,
  PostSavedForm,
};

impl Crud<PostForm> for Post {
  fn read(conn: &PgConnection, post_id: i32) -> Result<Self, Error> {
    use lemmy_db_schema::schema::post::dsl::*;
    post.find(post_id).first::<Self>(conn)
  }

  fn delete(conn: &PgConnection, post_id: i32) -> Result<usize, Error> {
    use lemmy_db_schema::schema::post::dsl::*;
    diesel::delete(post.find(post_id)).execute(conn)
  }

  fn create(conn: &PgConnection, new_post: &PostForm) -> Result<Self, Error> {
    use lemmy_db_schema::schema::post::dsl::*;
    insert_into(post).values(new_post).get_result::<Self>(conn)
  }

  fn update(conn: &PgConnection, post_id: i32, new_post: &PostForm) -> Result<Self, Error> {
    use lemmy_db_schema::schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set(new_post)
      .get_result::<Self>(conn)
  }
}

impl ApubObject<PostForm> for Post {
  fn read_from_apub_id(conn: &PgConnection, object_id: &str) -> Result<Self, Error> {
    use lemmy_db_schema::schema::post::dsl::*;
    post.filter(ap_id.eq(object_id)).first::<Self>(conn)
  }

  fn upsert(conn: &PgConnection, post_form: &PostForm) -> Result<Post, Error> {
    use lemmy_db_schema::schema::post::dsl::*;
    insert_into(post)
      .values(post_form)
      .on_conflict(ap_id)
      .do_update()
      .set(post_form)
      .get_result::<Self>(conn)
  }
}

impl Likeable<PostLikeForm> for PostLike {
  fn like(conn: &PgConnection, post_like_form: &PostLikeForm) -> Result<Self, Error> {
    use lemmy_db_schema::schema::post_like::dsl::*;
    insert_into(post_like)
      .values(post_like_form)
      .on_conflict((post_id, user_id))
      .do_update()
      .set(post_like_form)
      .get_result::<Self>(conn)
  }
  fn remove(conn: &PgConnection, user_id: i32, post_id: i32) -> Result<usize, Error> {
    use lemmy_db_schema::schema::post_like::dsl;
    diesel::delete(
      dsl::post_like
        .filter(dsl::post_id.eq(post_id))
        .filter(dsl::user_id.eq(user_id)),
    )
    .execute(conn)
  }
}

impl Saveable<PostSavedForm> for PostSaved {
  fn save(conn: &PgConnection, post_saved_form: &PostSavedForm) -> Result<Self, Error> {
    use lemmy_db_schema::schema::post_saved::dsl::*;
    insert_into(post_saved)
      .values(post_saved_form)
      .on_conflict((post_id, user_id))
      .do_update()
      .set(post_saved_form)
      .get_result::<Self>(conn)
  }
  fn unsave(conn: &PgConnection, post_saved_form: &PostSavedForm) -> Result<usize, Error> {
    use lemmy_db_schema::schema::post_saved::dsl::*;
    diesel::delete(
      post_saved
        .filter(post_id.eq(post_saved_form.post_id))
        .filter(user_id.eq(post_saved_form.user_id)),
    )
    .execute(conn)
  }
}

impl Readable<PostReadForm> for PostRead {
  fn mark_as_read(conn: &PgConnection, post_read_form: &PostReadForm) -> Result<Self, Error> {
    use lemmy_db_schema::schema::post_read::dsl::*;
    insert_into(post_read)
      .values(post_read_form)
      .get_result::<Self>(conn)
  }

  fn mark_as_unread(conn: &PgConnection, post_read_form: &PostReadForm) -> Result<usize, Error> {
    use lemmy_db_schema::schema::post_read::dsl::*;
    diesel::delete(
      post_read
        .filter(post_id.eq(post_read_form.post_id))
        .filter(user_id.eq(post_read_form.user_id)),
    )
    .execute(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    source::{community::*, post::*, user::*},
    tests::establish_unpooled_connection,
    ListingType,
    SortType,
  };

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_user = UserForm {
      name: "jim".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      matrix_user_id: None,
      avatar: None,
      banner: None,
      admin: false,
      banned: Some(false),
      published: None,
      updated: None,
      show_nsfw: false,
      theme: "browser".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
      actor_id: None,
      bio: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
    };

    let inserted_user = User_::create(&conn, &new_user).unwrap();

    let new_community = CommunityForm {
      name: "test community_3".to_string(),
      title: "nada".to_owned(),
      description: None,
      category_id: 1,
      creator_id: inserted_user.id,
      removed: None,
      deleted: None,
      updated: None,
      nsfw: false,
      actor_id: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
      published: None,
      icon: None,
      banner: None,
    };

    let inserted_community = Community::create(&conn, &new_community).unwrap();

    let new_post = PostForm {
      name: "A test post".into(),
      url: None,
      body: None,
      creator_id: inserted_user.id,
      community_id: inserted_community.id,
      removed: None,
      deleted: None,
      locked: None,
      stickied: None,
      nsfw: false,
      updated: None,
      embed_title: None,
      embed_description: None,
      embed_html: None,
      thumbnail_url: None,
      ap_id: None,
      local: true,
      published: None,
    };

    let inserted_post = Post::create(&conn, &new_post).unwrap();

    let expected_post = Post {
      id: inserted_post.id,
      name: "A test post".into(),
      url: None,
      body: None,
      creator_id: inserted_user.id,
      community_id: inserted_community.id,
      published: inserted_post.published,
      removed: false,
      locked: false,
      stickied: false,
      nsfw: false,
      deleted: false,
      updated: None,
      embed_title: None,
      embed_description: None,
      embed_html: None,
      thumbnail_url: None,
      ap_id: inserted_post.ap_id.to_owned(),
      local: true,
    };

    // Post Like
    let post_like_form = PostLikeForm {
      post_id: inserted_post.id,
      user_id: inserted_user.id,
      score: 1,
    };

    let inserted_post_like = PostLike::like(&conn, &post_like_form).unwrap();

    let expected_post_like = PostLike {
      id: inserted_post_like.id,
      post_id: inserted_post.id,
      user_id: inserted_user.id,
      published: inserted_post_like.published,
      score: 1,
    };

    // Post Save
    let post_saved_form = PostSavedForm {
      post_id: inserted_post.id,
      user_id: inserted_user.id,
    };

    let inserted_post_saved = PostSaved::save(&conn, &post_saved_form).unwrap();

    let expected_post_saved = PostSaved {
      id: inserted_post_saved.id,
      post_id: inserted_post.id,
      user_id: inserted_user.id,
      published: inserted_post_saved.published,
    };

    // Post Read
    let post_read_form = PostReadForm {
      post_id: inserted_post.id,
      user_id: inserted_user.id,
    };

    let inserted_post_read = PostRead::mark_as_read(&conn, &post_read_form).unwrap();

    let expected_post_read = PostRead {
      id: inserted_post_read.id,
      post_id: inserted_post.id,
      user_id: inserted_user.id,
      published: inserted_post_read.published,
    };

    let read_post = Post::read(&conn, inserted_post.id).unwrap();
    let updated_post = Post::update(&conn, inserted_post.id, &new_post).unwrap();
    let like_removed = PostLike::remove(&conn, inserted_user.id, inserted_post.id).unwrap();
    let saved_removed = PostSaved::unsave(&conn, &post_saved_form).unwrap();
    let read_removed = PostRead::mark_as_unread(&conn, &post_read_form).unwrap();
    let num_deleted = Post::delete(&conn, inserted_post.id).unwrap();
    Community::delete(&conn, inserted_community.id).unwrap();
    User_::delete(&conn, inserted_user.id).unwrap();

    assert_eq!(expected_post, read_post);
    assert_eq!(expected_post, inserted_post);
    assert_eq!(expected_post, updated_post);
    assert_eq!(expected_post_like, inserted_post_like);
    assert_eq!(expected_post_saved, inserted_post_saved);
    assert_eq!(expected_post_read, inserted_post_read);
    assert_eq!(1, like_removed);
    assert_eq!(1, saved_removed);
    assert_eq!(1, read_removed);
    assert_eq!(1, num_deleted);
  }
}
