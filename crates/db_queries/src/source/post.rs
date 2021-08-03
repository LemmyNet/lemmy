use crate::{ApubObject, Crud, DbPool, TokioDieselFuture, DeleteableOrRemoveable, Likeable, Readable, Saveable};
use tokio_diesel::*;
use diesel::{dsl::*, result::Error, *};
use lemmy_db_schema::{
  naive_now,
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
  CommunityId,
  DbUrl,
  PersonId,
  PostId,
};

impl<'a> Crud<'a, PostForm, PostId> for Post {
  fn read(pool: &'a DbPool, post_id: PostId) -> TokioDieselFuture<'a, Self> {
    use lemmy_db_schema::schema::post::dsl::*;
    post.find(post_id).first_async(pool)
  }

  fn delete(pool: &'a DbPool, post_id: PostId) -> TokioDieselFuture<'a, usize> {
    use lemmy_db_schema::schema::post::dsl::*;
    diesel::delete(post.find(post_id)).execute_async(pool)
  }

  fn create(pool: &'a DbPool, new_post: &'a PostForm) -> TokioDieselFuture<'a, Self> {
    use lemmy_db_schema::schema::post::dsl::*;
    insert_into(post).values(new_post).get_result_async(pool)
  }

  fn update(pool: &'a DbPool, post_id: PostId, new_post: &'a PostForm) -> TokioDieselFuture<'a, Self> {
    use lemmy_db_schema::schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set(new_post)
      .get_result_async(pool)
  }
}

pub trait Post_<'a>{
  fn list_for_community(
    pool: &'a DbPool,
    the_community_id: CommunityId,
  ) -> TokioDieselFuture<'a, Vec<Post>>;
  fn update_ap_id(pool: &'a DbPool, post_id: PostId, apub_id: DbUrl) -> TokioDieselFuture<'a, Post>;
  fn permadelete_for_creator(
    pool: &'a DbPool,
    for_creator_id: PersonId,
  ) -> TokioDieselFuture<'a, Vec<Post>>;
  fn update_deleted(pool: &'a DbPool, post_id: PostId, new_deleted: bool)
    -> TokioDieselFuture<'a, Post>;
  fn update_removed(pool: &'a DbPool, post_id: PostId, new_removed: bool)
    -> TokioDieselFuture<'a, Post>;
  fn update_removed_for_creator(
    pool: &'a DbPool,
    for_creator_id: PersonId,
    for_community_id: Option<CommunityId>,
    new_removed: bool,
  ) -> TokioDieselFuture<'a, Vec<Post>>;
  fn update_locked(pool: &'a DbPool, post_id: PostId, new_locked: bool) -> TokioDieselFuture<'a, Post>;
  fn update_stickied(
    pool: &'a DbPool,
    post_id: PostId,
    new_stickied: bool,
  ) -> TokioDieselFuture<'a, Post>;
  fn is_post_creator(person_id: PersonId, post_creator_id: PersonId) -> bool;
}

impl<'a> Post_<'a> for Post {
  fn list_for_community(
    pool: &'a DbPool,
    the_community_id: CommunityId,
  ) -> TokioDieselFuture<'a, Vec<Self>> {
    use lemmy_db_schema::schema::post::dsl::*;
    post
      .filter(community_id.eq(the_community_id))
      .then_order_by(published.desc())
      .then_order_by(stickied.desc())
      .limit(20)
      .load_async(pool)
  }

  fn update_ap_id(pool: &'a DbPool, post_id: PostId, apub_id: DbUrl) -> TokioDieselFuture<'a, Self> {
    use lemmy_db_schema::schema::post::dsl::*;

    diesel::update(post.find(post_id))
      .set(ap_id.eq(apub_id))
      .get_result_async(pool)
  }

  fn permadelete_for_creator(
    pool: &'a DbPool,
    for_creator_id: PersonId,
  ) -> TokioDieselFuture<'a, Vec<Self>> {
    use lemmy_db_schema::schema::post::dsl::*;

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
      .get_results_async(pool)
  }

  fn update_deleted(
    pool: &'a DbPool,
    post_id: PostId,
    new_deleted: bool,
  ) -> TokioDieselFuture<'a, Self> {
    use lemmy_db_schema::schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set((deleted.eq(new_deleted), updated.eq(naive_now())))
      .get_result_async(pool)
  }

  fn update_removed(
    pool: &'a DbPool,
    post_id: PostId,
    new_removed: bool,
  ) -> TokioDieselFuture<'a, Self> {
    use lemmy_db_schema::schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set((removed.eq(new_removed), updated.eq(naive_now())))
      .get_result_async(pool)
  }

  fn update_removed_for_creator(
    pool: &'a DbPool,
    for_creator_id: PersonId,
    for_community_id: Option<CommunityId>,
    new_removed: bool,
  ) -> TokioDieselFuture<'a, Vec<Self>> {
    use lemmy_db_schema::schema::post::dsl::*;

    let mut update = diesel::update(post).into_boxed();
    update = update.filter(creator_id.eq(for_creator_id));

    if let Some(for_community_id) = for_community_id {
      update = update.filter(community_id.eq(for_community_id));
    }

    update
      .set((removed.eq(new_removed), updated.eq(naive_now())))
      .get_results_async(pool)
  }

  fn update_locked(pool: &'a DbPool, post_id: PostId, new_locked: bool) -> TokioDieselFuture<'a, Self> {
    use lemmy_db_schema::schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set(locked.eq(new_locked))
      .get_result_async(pool)
  }

  fn update_stickied(
    pool: &'a DbPool,
    post_id: PostId,
    new_stickied: bool,
  ) -> TokioDieselFuture<'a, Self> {
    use lemmy_db_schema::schema::post::dsl::*;
    diesel::update(post.find(post_id))
      .set(stickied.eq(new_stickied))
      .get_result_async(pool)
  }

  fn is_post_creator(person_id: PersonId, post_creator_id: PersonId) -> bool {
    person_id == post_creator_id
  }
}

impl<'a> ApubObject<'a, PostForm> for Post {
  fn read_from_apub_id(pool: &'a DbPool, object_id: &'a DbUrl) -> TokioDieselFuture<'a, Self> {
    use lemmy_db_schema::schema::post::dsl::*;
    post.filter(ap_id.eq(object_id)).first_async(pool)
  }

  fn upsert(pool: &'a DbPool, post_form: &'a PostForm) -> TokioDieselFuture<'a, Post> {
    use lemmy_db_schema::schema::post::dsl::*;
    insert_into(post)
      .values(post_form)
      .on_conflict(ap_id)
      .do_update()
      .set(post_form)
      .get_result_async(pool)
  }
}

impl<'a> Likeable<'a, PostLikeForm, PostId> for PostLike {
  fn like(pool: &'a DbPool, post_like_form: &'a PostLikeForm) -> TokioDieselFuture<'a, Self> {
    use lemmy_db_schema::schema::post_like::dsl::*;
    insert_into(post_like)
      .values(post_like_form)
      .on_conflict((post_id, person_id))
      .do_update()
      .set(post_like_form)
      .get_result_async(pool)
  }
  fn remove(pool: &'a DbPool, person_id: PersonId, post_id: PostId) -> TokioDieselFuture<'a, usize> {
    use lemmy_db_schema::schema::post_like::dsl;
    diesel::delete(
      dsl::post_like
        .filter(dsl::post_id.eq(post_id))
        .filter(dsl::person_id.eq(person_id)),
    )
    .execute_async(pool)
  }
}

impl<'a> Saveable<'a, PostSavedForm> for PostSaved {
  fn save(pool: &'a DbPool, post_saved_form: &'a PostSavedForm) -> TokioDieselFuture<'a, Self> {
    use lemmy_db_schema::schema::post_saved::dsl::*;
    insert_into(post_saved)
      .values(post_saved_form)
      .on_conflict((post_id, person_id))
      .do_update()
      .set(post_saved_form)
      .get_result_async(pool)
  }
  fn unsave(pool: &'a DbPool, post_saved_form: &PostSavedForm) -> TokioDieselFuture<'a, usize> {
    use lemmy_db_schema::schema::post_saved::dsl::*;
    diesel::delete(
      post_saved
        .filter(post_id.eq(post_saved_form.post_id))
        .filter(person_id.eq(post_saved_form.person_id)),
    )
    .execute_async(pool)
  }
}

impl<'a> Readable<'a, PostReadForm> for PostRead {
  fn mark_as_read(pool: &'a DbPool, post_read_form: &'a PostReadForm) -> TokioDieselFuture<'a, Self> {
    use lemmy_db_schema::schema::post_read::dsl::*;
    insert_into(post_read)
      .values(post_read_form)
      .on_conflict((post_id, person_id))
      .do_update()
      .set(post_read_form)
      .get_result_async(pool)
  }

  fn mark_as_unread(pool: &'a DbPool, post_read_form: &'a PostReadForm) -> TokioDieselFuture<'a, usize> {
    use lemmy_db_schema::schema::post_read::dsl::*;
    diesel::delete(
      post_read
        .filter(post_id.eq(post_read_form.post_id))
        .filter(person_id.eq(post_read_form.person_id)),
    )
    .execute_async(pool)
  }
}

impl DeleteableOrRemoveable for Post {
  fn blank_out_deleted_or_removed_info(mut self) -> Self {
    self.name = "".into();
    self.url = None;
    self.body = None;
    self.embed_title = None;
    self.embed_description = None;
    self.embed_html = None;
    self.thumbnail_url = None;

    self
  }
}

#[cfg(test)]
mod tests {
  use crate::{setup_connection_pool_for_tests, source::post::*};
  use lemmy_db_schema::source::{
    community::{Community, CommunityForm},
    person::*,
  };

  #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
  async fn test_crud() {
    let pool = setup_connection_pool_for_tests();

    let new_person = PersonForm {
      name: "jim".into(),
      ..PersonForm::default()
    };

    let inserted_person = Person::create(&pool, &new_person).await.unwrap();

    let new_community = CommunityForm {
      name: "test community_3".to_string(),
      title: "nada".to_owned(),
      ..CommunityForm::default()
    };

    let inserted_community = Community::create(&pool, &new_community).await.unwrap();

    let new_post = PostForm {
      name: "A test post".into(),
      creator_id: inserted_person.id,
      community_id: inserted_community.id,
      ..PostForm::default()
    };

    let inserted_post = Post::create(&pool, &new_post).await.unwrap();

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
      embed_html: None,
      thumbnail_url: None,
      ap_id: inserted_post.ap_id.to_owned(),
      local: true,
    };

    // Post Like
    let post_like_form = PostLikeForm {
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      score: 1,
    };

    let inserted_post_like = PostLike::like(&pool, &post_like_form).await.unwrap();

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

    let inserted_post_saved = PostSaved::save(&pool, &post_saved_form).await.unwrap();

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

    let inserted_post_read = PostRead::mark_as_read(&pool, &post_read_form).await.unwrap();

    let expected_post_read = PostRead {
      id: inserted_post_read.id,
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      published: inserted_post_read.published,
    };

    let read_post = Post::read(&pool, inserted_post.id).await.unwrap();
    let updated_post = Post::update(&pool, inserted_post.id, &new_post).await.unwrap();
    let like_removed = PostLike::remove(&pool, inserted_person.id, inserted_post.id).await.unwrap();
    let saved_removed = PostSaved::unsave(&pool, &post_saved_form).await.unwrap();
    let read_removed = PostRead::mark_as_unread(&pool, &post_read_form).await.unwrap();
    let num_deleted = Post::delete(&pool, inserted_post.id).await.unwrap();
    Community::delete(&pool, inserted_community.id).await.unwrap();
    Person::delete(&pool, inserted_person.id).await.unwrap();

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
