use crate::{
  newtypes::{CommunityId, DbUrl, PersonId, PostId},
  schema::post::dsl::{
    ap_id,
    body,
    community_id,
    creator_id,
    deleted,
    featured_community,
    name,
    post,
    published,
    removed,
    thumbnail_url,
    updated,
    url,
  },
  source::post::{
    Post,
    PostInsertForm,
    PostLike,
    PostLikeForm,
    PostRead,
    PostReadForm,
    PostSaved,
    PostSavedForm,
    PostUpdateForm,
  },
  traits::{Crud, Likeable, Readable, Saveable},
  utils::{naive_now, DbConn, FETCH_LIMIT_MAX},
};
use ::url::Url;
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl, TextExpressionMethods};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Crud for Post {
  type InsertForm = PostInsertForm;
  type UpdateForm = PostUpdateForm;
  type IdType = PostId;
  async fn read(conn: &mut DbConn, post_id: PostId) -> Result<Self, Error> {
    post.find(post_id).first::<Self>(conn).await
  }

  async fn delete(conn: &mut DbConn, post_id: PostId) -> Result<usize, Error> {
    diesel::delete(post.find(post_id)).execute(conn).await
  }

  async fn create(conn: &mut DbConn, form: &Self::InsertForm) -> Result<Self, Error> {
    insert_into(post)
      .values(form)
      .on_conflict(ap_id)
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    conn: &mut DbConn,
    post_id: PostId,
    new_post: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    diesel::update(post.find(post_id))
      .set(new_post)
      .get_result::<Self>(conn)
      .await
  }
}

impl Post {
  pub async fn list_for_community(
    conn: &mut DbConn,
    the_community_id: CommunityId,
  ) -> Result<Vec<Self>, Error> {
    post
      .filter(community_id.eq(the_community_id))
      .filter(deleted.eq(false))
      .filter(removed.eq(false))
      .then_order_by(featured_community.desc())
      .then_order_by(published.desc())
      .limit(FETCH_LIMIT_MAX)
      .load::<Self>(conn)
      .await
  }

  pub async fn list_featured_for_community(
    conn: &mut DbConn,
    the_community_id: CommunityId,
  ) -> Result<Vec<Self>, Error> {
    post
      .filter(community_id.eq(the_community_id))
      .filter(deleted.eq(false))
      .filter(removed.eq(false))
      .filter(featured_community.eq(true))
      .then_order_by(published.desc())
      .limit(FETCH_LIMIT_MAX)
      .load::<Self>(conn)
      .await
  }

  pub async fn permadelete_for_creator(
    conn: &mut DbConn,
    for_creator_id: PersonId,
  ) -> Result<Vec<Self>, Error> {
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
      .await
  }

  pub async fn update_removed_for_creator(
    conn: &mut DbConn,
    for_creator_id: PersonId,
    for_community_id: Option<CommunityId>,
    new_removed: bool,
  ) -> Result<Vec<Self>, Error> {
    let mut update = diesel::update(post).into_boxed();
    update = update.filter(creator_id.eq(for_creator_id));

    if let Some(for_community_id) = for_community_id {
      update = update.filter(community_id.eq(for_community_id));
    }

    update
      .set((removed.eq(new_removed), updated.eq(naive_now())))
      .get_results::<Self>(conn)
      .await
  }

  pub fn is_post_creator(person_id: PersonId, post_creator_id: PersonId) -> bool {
    person_id == post_creator_id
  }

  pub async fn read_from_apub_id(conn: &mut DbConn, object_id: Url) -> Result<Option<Self>, Error> {
    let object_id: DbUrl = object_id.into();
    Ok(
      post
        .filter(ap_id.eq(object_id))
        .first::<Post>(conn)
        .await
        .ok()
        .map(Into::into),
    )
  }

  pub async fn fetch_pictrs_posts_for_creator(
    conn: &mut DbConn,
    for_creator_id: PersonId,
  ) -> Result<Vec<Self>, Error> {
    let pictrs_search = "%pictrs/image%";

    post
      .filter(creator_id.eq(for_creator_id))
      .filter(url.like(pictrs_search))
      .load::<Self>(conn)
      .await
  }

  /// Sets the url and thumbnails fields to None
  pub async fn remove_pictrs_post_images_and_thumbnails_for_creator(
    conn: &mut DbConn,
    for_creator_id: PersonId,
  ) -> Result<Vec<Self>, Error> {
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
    .await
  }

  pub async fn fetch_pictrs_posts_for_community(
    conn: &mut DbConn,
    for_community_id: CommunityId,
  ) -> Result<Vec<Self>, Error> {
    let pictrs_search = "%pictrs/image%";
    post
      .filter(community_id.eq(for_community_id))
      .filter(url.like(pictrs_search))
      .load::<Self>(conn)
      .await
  }

  /// Sets the url and thumbnails fields to None
  pub async fn remove_pictrs_post_images_and_thumbnails_for_community(
    conn: &mut DbConn,
    for_community_id: CommunityId,
  ) -> Result<Vec<Self>, Error> {
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
    .await
  }
}

#[async_trait]
impl Likeable for PostLike {
  type Form = PostLikeForm;
  type IdType = PostId;
  async fn like(conn: &mut DbConn, post_like_form: &PostLikeForm) -> Result<Self, Error> {
    use crate::schema::post_like::dsl::{person_id, post_id, post_like};
    insert_into(post_like)
      .values(post_like_form)
      .on_conflict((post_id, person_id))
      .do_update()
      .set(post_like_form)
      .get_result::<Self>(conn)
      .await
  }
  async fn remove(conn: &mut DbConn, person_id: PersonId, post_id: PostId) -> Result<usize, Error> {
    use crate::schema::post_like::dsl;
    diesel::delete(
      dsl::post_like
        .filter(dsl::post_id.eq(post_id))
        .filter(dsl::person_id.eq(person_id)),
    )
    .execute(conn)
    .await
  }
}

#[async_trait]
impl Saveable for PostSaved {
  type Form = PostSavedForm;
  async fn save(conn: &mut DbConn, post_saved_form: &PostSavedForm) -> Result<Self, Error> {
    use crate::schema::post_saved::dsl::{person_id, post_id, post_saved};
    insert_into(post_saved)
      .values(post_saved_form)
      .on_conflict((post_id, person_id))
      .do_update()
      .set(post_saved_form)
      .get_result::<Self>(conn)
      .await
  }
  async fn unsave(conn: &mut DbConn, post_saved_form: &PostSavedForm) -> Result<usize, Error> {
    use crate::schema::post_saved::dsl::{person_id, post_id, post_saved};
    diesel::delete(
      post_saved
        .filter(post_id.eq(post_saved_form.post_id))
        .filter(person_id.eq(post_saved_form.person_id)),
    )
    .execute(conn)
    .await
  }
}

#[async_trait]
impl Readable for PostRead {
  type Form = PostReadForm;
  async fn mark_as_read(conn: &mut DbConn, post_read_form: &PostReadForm) -> Result<Self, Error> {
    use crate::schema::post_read::dsl::{person_id, post_id, post_read};
    insert_into(post_read)
      .values(post_read_form)
      .on_conflict((post_id, person_id))
      .do_update()
      .set(post_read_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn mark_as_unread(
    conn: &mut DbConn,
    post_read_form: &PostReadForm,
  ) -> Result<usize, Error> {
    use crate::schema::post_read::dsl::{person_id, post_id, post_read};
    diesel::delete(
      post_read
        .filter(post_id.eq(post_read_form.post_id))
        .filter(person_id.eq(post_read_form.person_id)),
    )
    .execute(conn)
    .await
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    source::{
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{
        Post,
        PostInsertForm,
        PostLike,
        PostLikeForm,
        PostRead,
        PostReadForm,
        PostSaved,
        PostSavedForm,
        PostUpdateForm,
      },
    },
    traits::{Crud, Likeable, Readable, Saveable},
    utils::build_db_conn_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let conn = &mut build_db_conn_for_tests().await;

    let inserted_instance = Instance::read_or_create(conn, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person = PersonInsertForm::builder()
      .name("jim".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(conn, &new_person).await.unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("test community_3".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(conn, &new_community).await.unwrap();

    let new_post = PostInsertForm::builder()
      .name("A test post".into())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post = Post::create(conn, &new_post).await.unwrap();

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
      nsfw: false,
      deleted: false,
      updated: None,
      embed_title: None,
      embed_description: None,
      embed_video_url: None,
      thumbnail_url: None,
      ap_id: inserted_post.ap_id.clone(),
      local: true,
      language_id: Default::default(),
      featured_community: false,
      featured_local: false,
    };

    // Post Like
    let post_like_form = PostLikeForm {
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      score: 1,
    };

    let inserted_post_like = PostLike::like(conn, &post_like_form).await.unwrap();

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

    let inserted_post_saved = PostSaved::save(conn, &post_saved_form).await.unwrap();

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

    let inserted_post_read = PostRead::mark_as_read(conn, &post_read_form).await.unwrap();

    let expected_post_read = PostRead {
      id: inserted_post_read.id,
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      published: inserted_post_read.published,
    };

    let read_post = Post::read(conn, inserted_post.id).await.unwrap();

    let new_post_update = PostUpdateForm::builder()
      .name(Some("A test post".into()))
      .build();
    let updated_post = Post::update(conn, inserted_post.id, &new_post_update)
      .await
      .unwrap();

    let like_removed = PostLike::remove(conn, inserted_person.id, inserted_post.id)
      .await
      .unwrap();
    let saved_removed = PostSaved::unsave(conn, &post_saved_form).await.unwrap();
    let read_removed = PostRead::mark_as_unread(conn, &post_read_form)
      .await
      .unwrap();
    let num_deleted = Post::delete(conn, inserted_post.id).await.unwrap();
    Community::delete(conn, inserted_community.id)
      .await
      .unwrap();
    Person::delete(conn, inserted_person.id).await.unwrap();
    Instance::delete(conn, inserted_instance.id).await.unwrap();

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
