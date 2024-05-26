use crate::{
  diesel::OptionalExtension,
  newtypes::{CommunityId, DbUrl, PersonId, PostId},
  schema::{post, post_actions},
  source::post::{
    Post,
    PostHide,
    PostHideForm,
    PostInsertForm,
    PostLike,
    PostLikeForm,
    PostRead,
    PostReadForm,
    PostSaved,
    PostSavedForm,
    PostUpdateForm,
  },
  traits::{Crud, Likeable, Saveable},
  utils::{
    functions::coalesce,
    get_conn,
    naive_now,
    now,
    uplete::{OrDelete, UpleteCount, UpleteTable},
    DbPool,
    DELETED_REPLACEMENT_TEXT,
    FETCH_LIMIT_MAX,
    SITEMAP_DAYS,
    SITEMAP_LIMIT,
  },
};
use ::url::Url;
use chrono::{DateTime, Utc};
use diesel::{
  dsl::{self, insert_into},
  result::Error,
  DecoratableTarget,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
  TextExpressionMethods,
};
use diesel_async::RunQueryDsl;
use std::collections::HashSet;

impl UpleteTable for post_actions::table {
  type EmptyRow = (
    post_actions::post_id,
    post_actions::person_id,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
    Option<i64>,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
    Option<i16>,
    Option<DateTime<Utc>>,
  );
}

#[async_trait]
impl Crud for Post {
  type InsertForm = PostInsertForm;
  type UpdateForm = PostUpdateForm;
  type IdType = PostId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    new_post: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(post::table.find(post_id))
      .set(new_post)
      .get_result::<Self>(conn)
      .await
  }
}

impl Post {
  pub async fn insert_apub(
    pool: &mut DbPool<'_>,
    timestamp: DateTime<Utc>,
    form: &PostInsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post::table)
      .values(form)
      .on_conflict(post::ap_id)
      .filter_target(coalesce(post::updated, post::published).lt(timestamp))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn list_featured_for_community(
    pool: &mut DbPool<'_>,
    the_community_id: CommunityId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    post::table
      .filter(post::community_id.eq(the_community_id))
      .filter(post::deleted.eq(false))
      .filter(post::removed.eq(false))
      .filter(post::featured_community.eq(true))
      .then_order_by(post::published.desc())
      .limit(FETCH_LIMIT_MAX)
      .load::<Self>(conn)
      .await
  }

  pub async fn list_for_sitemap(
    pool: &mut DbPool<'_>,
  ) -> Result<Vec<(DbUrl, chrono::DateTime<Utc>)>, Error> {
    let conn = &mut get_conn(pool).await?;
    post::table
      .select((post::ap_id, coalesce(post::updated, post::published)))
      .filter(post::local.eq(true))
      .filter(post::deleted.eq(false))
      .filter(post::removed.eq(false))
      .filter(
        post::published.ge(Utc::now().naive_utc() - SITEMAP_DAYS.expect("TimeDelta out of bounds")),
      )
      .order(post::published.desc())
      .limit(SITEMAP_LIMIT)
      .load::<(DbUrl, chrono::DateTime<Utc>)>(conn)
      .await
  }

  pub async fn permadelete_for_creator(
    pool: &mut DbPool<'_>,
    for_creator_id: PersonId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;

    diesel::update(post::table.filter(post::creator_id.eq(for_creator_id)))
      .set((
        post::name.eq(DELETED_REPLACEMENT_TEXT),
        post::url.eq(Option::<&str>::None),
        post::body.eq(DELETED_REPLACEMENT_TEXT),
        post::deleted.eq(true),
        post::updated.eq(naive_now()),
      ))
      .get_results::<Self>(conn)
      .await
  }

  pub async fn update_removed_for_creator(
    pool: &mut DbPool<'_>,
    for_creator_id: PersonId,
    for_community_id: Option<CommunityId>,
    new_removed: bool,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;

    let mut update = diesel::update(post::table).into_boxed();
    update = update.filter(post::creator_id.eq(for_creator_id));

    if let Some(for_community_id) = for_community_id {
      update = update.filter(post::community_id.eq(for_community_id));
    }

    update
      .set((post::removed.eq(new_removed), post::updated.eq(naive_now())))
      .get_results::<Self>(conn)
      .await
  }

  pub fn is_post_creator(person_id: PersonId, post_creator_id: PersonId) -> bool {
    person_id == post_creator_id
  }

  pub async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: Url,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let object_id: DbUrl = object_id.into();
    post::table
      .filter(post::ap_id.eq(object_id))
      .first(conn)
      .await
      .optional()
  }

  pub async fn fetch_pictrs_posts_for_creator(
    pool: &mut DbPool<'_>,
    for_creator_id: PersonId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let pictrs_search = "%pictrs/image%";

    post::table
      .filter(post::creator_id.eq(for_creator_id))
      .filter(post::url.like(pictrs_search))
      .load::<Self>(conn)
      .await
  }

  /// Sets the url and thumbnails fields to None
  pub async fn remove_pictrs_post_images_and_thumbnails_for_creator(
    pool: &mut DbPool<'_>,
    for_creator_id: PersonId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let pictrs_search = "%pictrs/image%";

    diesel::update(
      post::table
        .filter(post::creator_id.eq(for_creator_id))
        .filter(post::url.like(pictrs_search)),
    )
    .set((
      post::url.eq::<Option<String>>(None),
      post::thumbnail_url.eq::<Option<String>>(None),
    ))
    .get_results::<Self>(conn)
    .await
  }

  pub async fn fetch_pictrs_posts_for_community(
    pool: &mut DbPool<'_>,
    for_community_id: CommunityId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let pictrs_search = "%pictrs/image%";
    post::table
      .filter(post::community_id.eq(for_community_id))
      .filter(post::url.like(pictrs_search))
      .load::<Self>(conn)
      .await
  }

  /// Sets the url and thumbnails fields to None
  pub async fn remove_pictrs_post_images_and_thumbnails_for_community(
    pool: &mut DbPool<'_>,
    for_community_id: CommunityId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let pictrs_search = "%pictrs/image%";

    diesel::update(
      post::table
        .filter(post::community_id.eq(for_community_id))
        .filter(post::url.like(pictrs_search)),
    )
    .set((
      post::url.eq::<Option<String>>(None),
      post::thumbnail_url.eq::<Option<String>>(None),
    ))
    .get_results::<Self>(conn)
    .await
  }
}

// TODO: replace `as_select_unwrap` definitions with `Default::default()` after next Diesel release
// which should implement `Default` for `AssumeNotNull`
impl PostLike {
  fn as_select_unwrap() -> (
    post_actions::post_id,
    post_actions::person_id,
    dsl::AssumeNotNull<post_actions::like_score>,
    dsl::AssumeNotNull<post_actions::liked>,
  ) {
    (
      post_actions::post_id,
      post_actions::person_id,
      post_actions::like_score.assume_not_null(),
      post_actions::liked.assume_not_null(),
    )
  }
}

#[async_trait]
impl Likeable for PostLike {
  type Form = PostLikeForm;
  type IdType = PostId;
  async fn like(pool: &mut DbPool<'_>, post_like_form: &PostLikeForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let post_like_form = (post_like_form, post_actions::liked.eq(now().nullable()));
    insert_into(post_actions::table)
      .values(post_like_form)
      .on_conflict((post_actions::post_id, post_actions::person_id))
      .do_update()
      .set(post_like_form)
      .returning(Self::as_select_unwrap())
      .get_result::<Self>(conn)
      .await
  }
  async fn remove(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    post_id: PostId,
  ) -> Result<UpleteCount, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(post_actions::table.find((person_id, post_id)))
      .set((
        post_actions::like_score.eq(None::<i16>),
        post_actions::liked.eq(None::<DateTime<Utc>>),
      ))
      .or_delete()
      .get_result(conn)
      .await
  }
}

impl PostSaved {
  fn as_select_unwrap() -> (
    post_actions::post_id,
    post_actions::person_id,
    dsl::AssumeNotNull<post_actions::saved>,
  ) {
    (
      post_actions::post_id,
      post_actions::person_id,
      post_actions::saved.assume_not_null(),
    )
  }
}

#[async_trait]
impl Saveable for PostSaved {
  type Form = PostSavedForm;
  async fn save(pool: &mut DbPool<'_>, post_saved_form: &PostSavedForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let post_saved_form = (post_saved_form, post_actions::saved.eq(now().nullable()));
    insert_into(post_actions::table)
      .values(post_saved_form)
      .on_conflict((post_actions::post_id, post_actions::person_id))
      .do_update()
      .set(post_saved_form)
      .returning(Self::as_select_unwrap())
      .get_result::<Self>(conn)
      .await
  }
  async fn unsave(
    pool: &mut DbPool<'_>,
    post_saved_form: &PostSavedForm,
  ) -> Result<UpleteCount, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(post_actions::table.find((post_saved_form.person_id, post_saved_form.post_id)))
      .set(post_actions::saved.eq(None::<DateTime<Utc>>))
      .or_delete()
      .get_result(conn)
      .await
  }
}

impl PostRead {
  pub async fn mark_as_read(
    pool: &mut DbPool<'_>,
    post_ids: HashSet<PostId>,
    person_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;

    let forms = post_ids
      .into_iter()
      .map(|post_id| {
        (
          PostReadForm { post_id, person_id },
          post_actions::read.eq(now().nullable()),
        )
      })
      .collect::<Vec<_>>();
    insert_into(post_actions::table)
      .values(forms)
      .on_conflict((post_actions::person_id, post_actions::post_id))
      .do_update()
      .set(post_actions::read.eq(now().nullable()))
      .execute(conn)
      .await
  }

  pub async fn mark_as_unread(
    pool: &mut DbPool<'_>,
    post_id_: HashSet<PostId>,
    person_id_: PersonId,
  ) -> Result<UpleteCount, Error> {
    let conn = &mut get_conn(pool).await?;

    diesel::update(
      post_actions::table
        .filter(post_actions::post_id.eq_any(post_id_))
        .filter(post_actions::person_id.eq(person_id_)),
    )
    .set(post_actions::read.eq(None::<DateTime<Utc>>))
    .or_delete()
    .get_result(conn)
    .await
  }
}

impl PostHide {
  pub async fn hide(
    pool: &mut DbPool<'_>,
    post_ids: HashSet<PostId>,
    person_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;

    let forms = post_ids
      .into_iter()
      .map(|post_id| {
        (
          PostHideForm { post_id, person_id },
          post_actions::hidden.eq(now().nullable()),
        )
      })
      .collect::<Vec<_>>();
    insert_into(post_actions::table)
      .values(forms)
      .on_conflict((post_actions::person_id, post_actions::post_id))
      .do_update()
      .set(post_actions::hidden.eq(now().nullable()))
      .execute(conn)
      .await
  }

  pub async fn unhide(
    pool: &mut DbPool<'_>,
    post_id_: HashSet<PostId>,
    person_id_: PersonId,
  ) -> Result<UpleteCount, Error> {
    let conn = &mut get_conn(pool).await?;

    diesel::update(
      post_actions::table
        .filter(post_actions::post_id.eq_any(post_id_))
        .filter(post_actions::person_id.eq(person_id_)),
    )
    .set(post_actions::hidden.eq(None::<DateTime<Utc>>))
    .or_delete()
    .get_result(conn)
    .await
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
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
        PostSaved,
        PostSavedForm,
        PostUpdateForm,
      },
    },
    traits::{Crud, Likeable, Saveable},
    utils::{build_db_pool_for_tests, uplete::UpleteCount},
  };
  use pretty_assertions::assert_eq;
  use serial_test::serial;
  use std::collections::HashSet;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person = PersonInsertForm::builder()
      .name("jim".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(pool, &new_person).await.unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("test community_3".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(pool, &new_community).await.unwrap();

    let new_post = PostInsertForm::builder()
      .name("A test post".into())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post = Post::create(pool, &new_post).await.unwrap();

    let new_post2 = PostInsertForm::builder()
      .name("A test post 2".into())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .build();
    let inserted_post2 = Post::create(pool, &new_post2).await.unwrap();

    let expected_post = Post {
      id: inserted_post.id,
      name: "A test post".into(),
      url: None,
      body: None,
      alt_text: None,
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
      url_content_type: None,
    };

    // Post Like
    let post_like_form = PostLikeForm {
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      score: 1,
    };

    let inserted_post_like = PostLike::like(pool, &post_like_form).await.unwrap();

    let expected_post_like = PostLike {
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

    let inserted_post_saved = PostSaved::save(pool, &post_saved_form).await.unwrap();

    let expected_post_saved = PostSaved {
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      published: inserted_post_saved.published,
    };

    // Post Read
    let marked_as_read = PostRead::mark_as_read(
      pool,
      HashSet::from([inserted_post.id, inserted_post2.id]),
      inserted_person.id,
    )
    .await
    .unwrap();
    assert_eq!(2, marked_as_read);

    let read_post = Post::read(pool, inserted_post.id).await.unwrap().unwrap();

    let new_post_update = PostUpdateForm {
      name: Some("A test post".into()),
      ..Default::default()
    };
    let updated_post = Post::update(pool, inserted_post.id, &new_post_update)
      .await
      .unwrap();

    let like_removed = PostLike::remove(pool, inserted_person.id, inserted_post.id)
      .await
      .unwrap();
    assert_eq!(UpleteCount::only_updated(1), like_removed);
    let saved_removed = PostSaved::unsave(pool, &post_saved_form).await.unwrap();
    assert_eq!(UpleteCount::only_updated(1), saved_removed);
    let read_removed = PostRead::mark_as_unread(
      pool,
      HashSet::from([inserted_post.id, inserted_post2.id]),
      inserted_person.id,
    )
    .await
    .unwrap();
    assert_eq!(UpleteCount::only_deleted(2), read_removed);

    let num_deleted = Post::delete(pool, inserted_post.id).await.unwrap()
      + Post::delete(pool, inserted_post2.id).await.unwrap();
    assert_eq!(2, num_deleted);
    Community::delete(pool, inserted_community.id)
      .await
      .unwrap();
    Person::delete(pool, inserted_person.id).await.unwrap();
    Instance::delete(pool, inserted_instance.id).await.unwrap();

    assert_eq!(expected_post, read_post);
    assert_eq!(expected_post, inserted_post);
    assert_eq!(expected_post, updated_post);
    assert_eq!(expected_post_like, inserted_post_like);
    assert_eq!(expected_post_saved, inserted_post_saved);
  }
}
