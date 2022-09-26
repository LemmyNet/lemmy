use crate::{
  newtypes::{CommunityId, DbUrl, PersonId, PostId},
  source::post::{
    Post,
    PostForm,
    PostLike,
    PostLikeForm,
    PostRead,
    PostReadForm,
    PostSaved,
    PostSavedForm,
  },
  traits::{Crud, DeleteableOrRemoveable, Likeable, Readable, Saveable},
  utils::{naive_now, FETCH_LIMIT_MAX},
};
use diesel::{dsl::*, result::Error, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, *};
use url::Url;

impl Crud for Post {
  type Form = PostForm;
  type IdType = PostId;
  fn read(conn: &mut PgConnection, post_id: PostId) -> Result<Self, Error> {
    use crate::schema::post::dsl::*;
    post.find(post_id).first::<Self>(conn)
  }

  fn delete(conn: &mut PgConnection, post_id: PostId) -> Result<usize, Error> {
    use crate::schema::post::dsl::*;
    diesel::delete(post.find(post_id)).execute(conn)
  }

  fn create(conn: &mut PgConnection, new_post: &PostForm) -> Result<Self, Error> {
    use crate::schema::post::dsl::*;
    insert_into(post).values(new_post).get_result::<Self>(conn)
  }

  fn update(conn: &mut PgConnection, post_id: PostId, new_post: &PostForm) -> Result<Self, Error> {
    use crate::schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set(new_post)
      .get_result::<Self>(conn)
  }
}

impl Post {
  pub fn list_for_community(
    conn: &mut PgConnection,
    the_community_id: CommunityId,
  ) -> Result<Vec<Self>, Error> {
    use crate::schema::post::dsl::*;
    post
      .filter(community_id.eq(the_community_id))
      .filter(deleted.eq(false))
      .filter(removed.eq(false))
      .then_order_by(published.desc())
      .then_order_by(stickied.desc())
      .limit(FETCH_LIMIT_MAX)
      .load::<Self>(conn)
  }

  pub fn update_ap_id(
    conn: &mut PgConnection,
    post_id: PostId,
    apub_id: DbUrl,
  ) -> Result<Self, Error> {
    use crate::schema::post::dsl::*;

    diesel::update(post.find(post_id))
      .set(ap_id.eq(apub_id))
      .get_result::<Self>(conn)
  }

  pub fn permadelete_for_creator(
    conn: &mut PgConnection,
    for_creator_id: PersonId,
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
    conn: &mut PgConnection,
    post_id: PostId,
    new_deleted: bool,
  ) -> Result<Self, Error> {
    use crate::schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set((deleted.eq(new_deleted), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  pub fn update_removed(
    conn: &mut PgConnection,
    post_id: PostId,
    new_removed: bool,
  ) -> Result<Self, Error> {
    use crate::schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set((removed.eq(new_removed), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  pub fn update_removed_for_creator(
    conn: &mut PgConnection,
    for_creator_id: PersonId,
    for_community_id: Option<CommunityId>,
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

  pub fn update_locked(
    conn: &mut PgConnection,
    post_id: PostId,
    new_locked: bool,
  ) -> Result<Self, Error> {
    use crate::schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set(locked.eq(new_locked))
      .get_result::<Self>(conn)
  }

  pub fn update_stickied(
    conn: &mut PgConnection,
    post_id: PostId,
    new_stickied: bool,
  ) -> Result<Self, Error> {
    use crate::schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set(stickied.eq(new_stickied))
      .get_result::<Self>(conn)
  }

  pub fn is_post_creator(person_id: PersonId, post_creator_id: PersonId) -> bool {
    person_id == post_creator_id
  }

  pub fn upsert(conn: &mut PgConnection, post_form: &PostForm) -> Result<Post, Error> {
    use crate::schema::post::dsl::*;
    insert_into(post)
      .values(post_form)
      .on_conflict(ap_id)
      .do_update()
      .set(post_form)
      .get_result::<Self>(conn)
  }
  pub fn read_from_apub_id(conn: &mut PgConnection, object_id: Url) -> Result<Option<Self>, Error> {
    use crate::schema::post::dsl::*;
    let object_id: DbUrl = object_id.into();
    Ok(
      post
        .filter(ap_id.eq(object_id))
        .first::<Post>(conn)
        .ok()
        .map(Into::into),
    )
  }

  pub fn fetch_pictrs_posts_for_creator(
    conn: &mut PgConnection,
    for_creator_id: PersonId,
  ) -> Result<Vec<Self>, Error> {
    use crate::schema::post::dsl::*;
    let pictrs_search = "%pictrs/image%";

    post
      .filter(creator_id.eq(for_creator_id))
      .filter(url.like(pictrs_search))
      .load::<Self>(conn)
  }

  /// Sets the url and thumbnails fields to None
  pub fn remove_pictrs_post_images_and_thumbnails_for_creator(
    conn: &mut PgConnection,
    for_creator_id: PersonId,
  ) -> Result<Vec<Self>, Error> {
    use crate::schema::post::dsl::*;
    let pictrs_search = "%pictrs/image%";

    diesel::update(
      post
        .filter(creator_id.eq(for_creator_id))
        .filter(url.like(pictrs_search)),
    )
    .set((
      url.eq::<Option<String>>(None),
      thumbnail_url.eq::<Option<String>>(None),
    ))
    .get_results::<Self>(conn)
  }

  pub fn fetch_pictrs_posts_for_community(
    conn: &mut PgConnection,
    for_community_id: CommunityId,
  ) -> Result<Vec<Self>, Error> {
    use crate::schema::post::dsl::*;
    let pictrs_search = "%pictrs/image%";
    post
      .filter(community_id.eq(for_community_id))
      .filter(url.like(pictrs_search))
      .load::<Self>(conn)
  }

  /// Sets the url and thumbnails fields to None
  pub fn remove_pictrs_post_images_and_thumbnails_for_community(
    conn: &mut PgConnection,
    for_community_id: CommunityId,
  ) -> Result<Vec<Self>, Error> {
    use crate::schema::post::dsl::*;
    let pictrs_search = "%pictrs/image%";

    diesel::update(
      post
        .filter(community_id.eq(for_community_id))
        .filter(url.like(pictrs_search)),
    )
    .set((
      url.eq::<Option<String>>(None),
      thumbnail_url.eq::<Option<String>>(None),
    ))
    .get_results::<Self>(conn)
  }
}

impl Likeable for PostLike {
  type Form = PostLikeForm;
  type IdType = PostId;
  fn like(conn: &mut PgConnection, post_like_form: &PostLikeForm) -> Result<Self, Error> {
    use crate::schema::post_like::dsl::*;
    insert_into(post_like)
      .values(post_like_form)
      .on_conflict((post_id, person_id))
      .do_update()
      .set(post_like_form)
      .get_result::<Self>(conn)
  }
  fn remove(conn: &mut PgConnection, person_id: PersonId, post_id: PostId) -> Result<usize, Error> {
    use crate::schema::post_like::dsl;
    diesel::delete(
      dsl::post_like
        .filter(dsl::post_id.eq(post_id))
        .filter(dsl::person_id.eq(person_id)),
    )
    .execute(conn)
  }
}

impl Saveable for PostSaved {
  type Form = PostSavedForm;
  fn save(conn: &mut PgConnection, post_saved_form: &PostSavedForm) -> Result<Self, Error> {
    use crate::schema::post_saved::dsl::*;
    insert_into(post_saved)
      .values(post_saved_form)
      .on_conflict((post_id, person_id))
      .do_update()
      .set(post_saved_form)
      .get_result::<Self>(conn)
  }
  fn unsave(conn: &mut PgConnection, post_saved_form: &PostSavedForm) -> Result<usize, Error> {
    use crate::schema::post_saved::dsl::*;
    diesel::delete(
      post_saved
        .filter(post_id.eq(post_saved_form.post_id))
        .filter(person_id.eq(post_saved_form.person_id)),
    )
    .execute(conn)
  }
}

impl Readable for PostRead {
  type Form = PostReadForm;
  fn mark_as_read(conn: &mut PgConnection, post_read_form: &PostReadForm) -> Result<Self, Error> {
    use crate::schema::post_read::dsl::*;
    insert_into(post_read)
      .values(post_read_form)
      .on_conflict((post_id, person_id))
      .do_update()
      .set(post_read_form)
      .get_result::<Self>(conn)
  }

  fn mark_as_unread(
    conn: &mut PgConnection,
    post_read_form: &PostReadForm,
  ) -> Result<usize, Error> {
    use crate::schema::post_read::dsl::*;
    diesel::delete(
      post_read
        .filter(post_id.eq(post_read_form.post_id))
        .filter(person_id.eq(post_read_form.person_id)),
    )
    .execute(conn)
  }
}

impl DeleteableOrRemoveable for Post {
  fn blank_out_deleted_or_removed_info(mut self) -> Self {
    self.name = "".into();
    self.url = None;
    self.body = None;
    self.embed_title = None;
    self.embed_description = None;
    self.embed_video_url = None;
    self.thumbnail_url = None;

    self
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    source::{
      community::{Community, CommunityForm},
      person::*,
      post::*,
    },
    traits::{Crud, Likeable, Readable, Saveable},
    utils::establish_unpooled_connection,
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = &mut establish_unpooled_connection();

    let new_person = PersonForm {
      name: "jim".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let inserted_person = Person::create(conn, &new_person).unwrap();

    let new_community = CommunityForm {
      name: "test community_3".to_string(),
      title: "nada".to_owned(),
      public_key: Some("pubkey".to_string()),
      ..CommunityForm::default()
    };

    let inserted_community = Community::create(conn, &new_community).unwrap();

    let new_post = PostForm {
      name: "A test post".into(),
      creator_id: inserted_person.id,
      community_id: inserted_community.id,
      ..PostForm::default()
    };

    let inserted_post = Post::create(conn, &new_post).unwrap();

    let expected_post = Post {
      id: inserted_post.id,
      name: "A test post".into(),
      url: None,
      body: None,
      creator_id: inserted_person.id,
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
      embed_video_url: None,
      thumbnail_url: None,
      ap_id: inserted_post.ap_id.to_owned(),
      local: true,
      language_id: Default::default(),
    };

    // Post Like
    let post_like_form = PostLikeForm {
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      score: 1,
    };

    let inserted_post_like = PostLike::like(conn, &post_like_form).unwrap();

    let expected_post_like = PostLike {
      id: inserted_post_like.id,
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      published: inserted_post_like.published,
      score: 1,
    };

    // Post Save
    let post_saved_form = PostSavedForm {
      post_id: inserted_post.id,
      person_id: inserted_person.id,
    };

    let inserted_post_saved = PostSaved::save(conn, &post_saved_form).unwrap();

    let expected_post_saved = PostSaved {
      id: inserted_post_saved.id,
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      published: inserted_post_saved.published,
    };

    // Post Read
    let post_read_form = PostReadForm {
      post_id: inserted_post.id,
      person_id: inserted_person.id,
    };

    let inserted_post_read = PostRead::mark_as_read(conn, &post_read_form).unwrap();

    let expected_post_read = PostRead {
      id: inserted_post_read.id,
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      published: inserted_post_read.published,
    };

    let read_post = Post::read(conn, inserted_post.id).unwrap();
    let updated_post = Post::update(conn, inserted_post.id, &new_post).unwrap();
    let like_removed = PostLike::remove(conn, inserted_person.id, inserted_post.id).unwrap();
    let saved_removed = PostSaved::unsave(conn, &post_saved_form).unwrap();
    let read_removed = PostRead::mark_as_unread(conn, &post_read_form).unwrap();
    let num_deleted = Post::delete(conn, inserted_post.id).unwrap();
    Community::delete(conn, inserted_community.id).unwrap();
    Person::delete(conn, inserted_person.id).unwrap();

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
