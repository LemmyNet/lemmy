use crate::{ApubObject, Crud, Likeable, Saveable};
use diesel::{dsl::*, result::Error, *};
use lemmy_db_schema::{
  naive_now,
  source::comment::{
    Comment,
    CommentForm,
    CommentLike,
    CommentLikeForm,
    CommentSaved,
    CommentSavedForm,
  },
  Url,
};

pub trait Comment_ {
  fn update_ap_id(conn: &PgConnection, comment_id: i32, apub_id: String) -> Result<Comment, Error>;
  fn permadelete_for_creator(
    conn: &PgConnection,
    for_creator_id: i32,
  ) -> Result<Vec<Comment>, Error>;
  fn update_deleted(
    conn: &PgConnection,
    comment_id: i32,
    new_deleted: bool,
  ) -> Result<Comment, Error>;
  fn update_removed(
    conn: &PgConnection,
    comment_id: i32,
    new_removed: bool,
  ) -> Result<Comment, Error>;
  fn update_removed_for_creator(
    conn: &PgConnection,
    for_creator_id: i32,
    new_removed: bool,
  ) -> Result<Vec<Comment>, Error>;
  fn update_read(conn: &PgConnection, comment_id: i32, new_read: bool) -> Result<Comment, Error>;
  fn update_content(
    conn: &PgConnection,
    comment_id: i32,
    new_content: &str,
  ) -> Result<Comment, Error>;
}

impl Comment_ for Comment {
  fn update_ap_id(conn: &PgConnection, comment_id: i32, apub_id: String) -> Result<Self, Error> {
    use lemmy_db_schema::schema::comment::dsl::*;

    diesel::update(comment.find(comment_id))
      .set(ap_id.eq(apub_id))
      .get_result::<Self>(conn)
  }

  fn permadelete_for_creator(conn: &PgConnection, for_creator_id: i32) -> Result<Vec<Self>, Error> {
    use lemmy_db_schema::schema::comment::dsl::*;
    diesel::update(comment.filter(creator_id.eq(for_creator_id)))
      .set((
        content.eq("*Permananently Deleted*"),
        deleted.eq(true),
        updated.eq(naive_now()),
      ))
      .get_results::<Self>(conn)
  }

  fn update_deleted(
    conn: &PgConnection,
    comment_id: i32,
    new_deleted: bool,
  ) -> Result<Self, Error> {
    use lemmy_db_schema::schema::comment::dsl::*;
    diesel::update(comment.find(comment_id))
      .set((deleted.eq(new_deleted), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  fn update_removed(
    conn: &PgConnection,
    comment_id: i32,
    new_removed: bool,
  ) -> Result<Self, Error> {
    use lemmy_db_schema::schema::comment::dsl::*;
    diesel::update(comment.find(comment_id))
      .set((removed.eq(new_removed), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  fn update_removed_for_creator(
    conn: &PgConnection,
    for_creator_id: i32,
    new_removed: bool,
  ) -> Result<Vec<Self>, Error> {
    use lemmy_db_schema::schema::comment::dsl::*;
    diesel::update(comment.filter(creator_id.eq(for_creator_id)))
      .set((removed.eq(new_removed), updated.eq(naive_now())))
      .get_results::<Self>(conn)
  }

  fn update_read(conn: &PgConnection, comment_id: i32, new_read: bool) -> Result<Self, Error> {
    use lemmy_db_schema::schema::comment::dsl::*;
    diesel::update(comment.find(comment_id))
      .set(read.eq(new_read))
      .get_result::<Self>(conn)
  }

  fn update_content(
    conn: &PgConnection,
    comment_id: i32,
    new_content: &str,
  ) -> Result<Self, Error> {
    use lemmy_db_schema::schema::comment::dsl::*;
    diesel::update(comment.find(comment_id))
      .set((content.eq(new_content), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }
}

impl Crud<CommentForm> for Comment {
  fn read(conn: &PgConnection, comment_id: i32) -> Result<Self, Error> {
    use lemmy_db_schema::schema::comment::dsl::*;
    comment.find(comment_id).first::<Self>(conn)
  }

  fn delete(conn: &PgConnection, comment_id: i32) -> Result<usize, Error> {
    use lemmy_db_schema::schema::comment::dsl::*;
    diesel::delete(comment.find(comment_id)).execute(conn)
  }

  fn create(conn: &PgConnection, comment_form: &CommentForm) -> Result<Self, Error> {
    use lemmy_db_schema::schema::comment::dsl::*;
    insert_into(comment)
      .values(comment_form)
      .get_result::<Self>(conn)
  }

  fn update(
    conn: &PgConnection,
    comment_id: i32,
    comment_form: &CommentForm,
  ) -> Result<Self, Error> {
    use lemmy_db_schema::schema::comment::dsl::*;
    diesel::update(comment.find(comment_id))
      .set(comment_form)
      .get_result::<Self>(conn)
  }
}

impl ApubObject<CommentForm> for Comment {
  fn read_from_apub_id(conn: &PgConnection, object_id: &Url) -> Result<Self, Error> {
    use lemmy_db_schema::schema::comment::dsl::*;
    comment.filter(ap_id.eq(object_id)).first::<Self>(conn)
  }

  fn upsert(conn: &PgConnection, comment_form: &CommentForm) -> Result<Self, Error> {
    use lemmy_db_schema::schema::comment::dsl::*;
    insert_into(comment)
      .values(comment_form)
      .on_conflict(ap_id)
      .do_update()
      .set(comment_form)
      .get_result::<Self>(conn)
  }
}

impl Likeable<CommentLikeForm> for CommentLike {
  fn like(conn: &PgConnection, comment_like_form: &CommentLikeForm) -> Result<Self, Error> {
    use lemmy_db_schema::schema::comment_like::dsl::*;
    insert_into(comment_like)
      .values(comment_like_form)
      .on_conflict((comment_id, user_id))
      .do_update()
      .set(comment_like_form)
      .get_result::<Self>(conn)
  }
  fn remove(conn: &PgConnection, user_id: i32, comment_id: i32) -> Result<usize, Error> {
    use lemmy_db_schema::schema::comment_like::dsl;
    diesel::delete(
      dsl::comment_like
        .filter(dsl::comment_id.eq(comment_id))
        .filter(dsl::user_id.eq(user_id)),
    )
    .execute(conn)
  }
}

impl Saveable<CommentSavedForm> for CommentSaved {
  fn save(conn: &PgConnection, comment_saved_form: &CommentSavedForm) -> Result<Self, Error> {
    use lemmy_db_schema::schema::comment_saved::dsl::*;
    insert_into(comment_saved)
      .values(comment_saved_form)
      .on_conflict((comment_id, user_id))
      .do_update()
      .set(comment_saved_form)
      .get_result::<Self>(conn)
  }
  fn unsave(conn: &PgConnection, comment_saved_form: &CommentSavedForm) -> Result<usize, Error> {
    use lemmy_db_schema::schema::comment_saved::dsl::*;
    diesel::delete(
      comment_saved
        .filter(comment_id.eq(comment_saved_form.comment_id))
        .filter(user_id.eq(comment_saved_form.user_id)),
    )
    .execute(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{establish_unpooled_connection, Crud, Likeable, ListingType, Saveable, SortType};
  use lemmy_db_schema::source::{
    comment::*,
    community::{Community, CommunityForm},
    post::*,
    user::{UserForm, User_},
  };

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_user = UserForm {
      name: "terry".into(),
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
      name: "test community".to_string(),
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
      banner: None,
      icon: None,
    };

    let inserted_community = Community::create(&conn, &new_community).unwrap();

    let new_post = PostForm {
      name: "A test post".into(),
      creator_id: inserted_user.id,
      url: None,
      body: None,
      community_id: inserted_community.id,
      removed: None,
      deleted: None,
      locked: None,
      stickied: None,
      updated: None,
      nsfw: false,
      embed_title: None,
      embed_description: None,
      embed_html: None,
      thumbnail_url: None,
      ap_id: None,
      local: true,
      published: None,
    };

    let inserted_post = Post::create(&conn, &new_post).unwrap();

    let comment_form = CommentForm {
      content: "A test comment".into(),
      creator_id: inserted_user.id,
      post_id: inserted_post.id,
      removed: None,
      deleted: None,
      read: None,
      parent_id: None,
      published: None,
      updated: None,
      ap_id: None,
      local: true,
    };

    let inserted_comment = Comment::create(&conn, &comment_form).unwrap();

    let expected_comment = Comment {
      id: inserted_comment.id,
      content: "A test comment".into(),
      creator_id: inserted_user.id,
      post_id: inserted_post.id,
      removed: false,
      deleted: false,
      read: false,
      parent_id: None,
      published: inserted_comment.published,
      updated: None,
      ap_id: inserted_comment.ap_id.to_owned(),
      local: true,
    };

    let child_comment_form = CommentForm {
      content: "A child comment".into(),
      creator_id: inserted_user.id,
      post_id: inserted_post.id,
      parent_id: Some(inserted_comment.id),
      removed: None,
      deleted: None,
      read: None,
      published: None,
      updated: None,
      ap_id: None,
      local: true,
    };

    let inserted_child_comment = Comment::create(&conn, &child_comment_form).unwrap();

    // Comment Like
    let comment_like_form = CommentLikeForm {
      comment_id: inserted_comment.id,
      post_id: inserted_post.id,
      user_id: inserted_user.id,
      score: 1,
    };

    let inserted_comment_like = CommentLike::like(&conn, &comment_like_form).unwrap();

    let expected_comment_like = CommentLike {
      id: inserted_comment_like.id,
      comment_id: inserted_comment.id,
      post_id: inserted_post.id,
      user_id: inserted_user.id,
      published: inserted_comment_like.published,
      score: 1,
    };

    // Comment Saved
    let comment_saved_form = CommentSavedForm {
      comment_id: inserted_comment.id,
      user_id: inserted_user.id,
    };

    let inserted_comment_saved = CommentSaved::save(&conn, &comment_saved_form).unwrap();

    let expected_comment_saved = CommentSaved {
      id: inserted_comment_saved.id,
      comment_id: inserted_comment.id,
      user_id: inserted_user.id,
      published: inserted_comment_saved.published,
    };

    let read_comment = Comment::read(&conn, inserted_comment.id).unwrap();
    let updated_comment = Comment::update(&conn, inserted_comment.id, &comment_form).unwrap();
    let like_removed = CommentLike::remove(&conn, inserted_user.id, inserted_comment.id).unwrap();
    let saved_removed = CommentSaved::unsave(&conn, &comment_saved_form).unwrap();
    let num_deleted = Comment::delete(&conn, inserted_comment.id).unwrap();
    Comment::delete(&conn, inserted_child_comment.id).unwrap();
    Post::delete(&conn, inserted_post.id).unwrap();
    Community::delete(&conn, inserted_community.id).unwrap();
    User_::delete(&conn, inserted_user.id).unwrap();

    assert_eq!(expected_comment, read_comment);
    assert_eq!(expected_comment, inserted_comment);
    assert_eq!(expected_comment, updated_comment);
    assert_eq!(expected_comment_like, inserted_comment_like);
    assert_eq!(expected_comment_saved, inserted_comment_saved);
    assert_eq!(
      expected_comment.id,
      inserted_child_comment.parent_id.unwrap()
    );
    assert_eq!(1, like_removed);
    assert_eq!(1, saved_removed);
    assert_eq!(1, num_deleted);
  }
}
