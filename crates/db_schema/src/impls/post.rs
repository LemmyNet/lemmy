use crate::{
  diesel::{BoolExpressionMethods, NullableExpressionMethods, OptionalExtension},
  newtypes::{CommunityId, DbUrl, PersonId, PostId},
  schema::{community, person, post, post_actions},
  source::post::{
    Post,
    PostActionsCursor,
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
    now,
    uplete,
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
  dsl::{count, insert_into, not},
  expression::SelectableHelper,
  result::Error,
  DecoratableTarget,
  ExpressionMethods,
  QueryDsl,
  TextExpressionMethods,
};
use diesel_async::RunQueryDsl;
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};

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
  pub async fn read_xx(pool: &mut DbPool<'_>, id: PostId) -> Result<Self, Error> {
    let conn = &mut *get_conn(pool).await?;
    post::table.find(id).first(conn).await
  }
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
      .filter(post::published.ge(Utc::now().naive_utc() - SITEMAP_DAYS))
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
        post::updated.eq(Utc::now()),
      ))
      .get_results::<Self>(conn)
      .await
  }

  pub async fn update_removed_for_creator(
    pool: &mut DbPool<'_>,
    for_creator_id: PersonId,
    for_community_id: Option<CommunityId>,
    removed: bool,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;

    let mut update = diesel::update(post::table).into_boxed();
    update = update.filter(post::creator_id.eq(for_creator_id));

    if let Some(for_community_id) = for_community_id {
      update = update.filter(post::community_id.eq(for_community_id));
    }

    update
      .set((post::removed.eq(removed), post::updated.eq(Utc::now())))
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
      .filter(post::scheduled_publish_time.is_null())
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

  pub async fn user_scheduled_post_count(
    person_id: PersonId,
    pool: &mut DbPool<'_>,
  ) -> Result<i64, Error> {
    let conn = &mut get_conn(pool).await?;

    post::table
      .inner_join(person::table)
      .inner_join(community::table)
      // find all posts which have scheduled_publish_time that is in the  future
      .filter(post::scheduled_publish_time.is_not_null())
      .filter(coalesce(post::scheduled_publish_time, now()).gt(now()))
      // make sure the post and community are still around
      .filter(not(post::deleted.or(post::removed)))
      .filter(not(community::removed.or(community::deleted)))
      // only posts by specified user
      .filter(post::creator_id.eq(person_id))
      .select(count(post::id))
      .first::<i64>(conn)
      .await
  }

  pub fn local_url(&self, settings: &Settings) -> LemmyResult<DbUrl> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/post/{}", self.id))?.into())
  }
}

#[async_trait]
impl Likeable for PostLike {
  type Form = PostLikeForm;
  type IdType = PostId;
  async fn like(pool: &mut DbPool<'_>, post_like_form: &PostLikeForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_actions::table)
      .values(post_like_form)
      .on_conflict((post_actions::post_id, post_actions::person_id))
      .do_update()
      .set(post_like_form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
  }
  async fn remove(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    post_id: PostId,
  ) -> Result<uplete::Count, Error> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(post_actions::table.find((person_id, post_id)))
      .set_null(post_actions::like_score)
      .set_null(post_actions::liked)
      .get_result(conn)
      .await
  }
}

#[async_trait]
impl Saveable for PostSaved {
  type Form = PostSavedForm;
  async fn save(pool: &mut DbPool<'_>, post_saved_form: &PostSavedForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_actions::table)
      .values(post_saved_form)
      .on_conflict((post_actions::post_id, post_actions::person_id))
      .do_update()
      .set(post_saved_form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
  }
  async fn unsave(
    pool: &mut DbPool<'_>,
    post_saved_form: &PostSavedForm,
  ) -> Result<uplete::Count, Error> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(post_actions::table.find((post_saved_form.person_id, post_saved_form.post_id)))
      .set_null(post_actions::saved)
      .get_result(conn)
      .await
  }
}

impl PostRead {
  pub async fn mark_as_read(
    pool: &mut DbPool<'_>,
    post_read_form: &PostReadForm,
  ) -> LemmyResult<usize> {
    Self::mark_many_as_read(pool, &[post_read_form.post_id], post_read_form.person_id).await
  }

  pub async fn mark_as_unread(
    pool: &mut DbPool<'_>,
    post_read_form: &PostReadForm,
  ) -> Result<uplete::Count, Error> {
    let conn = &mut get_conn(pool).await?;

    uplete::new(
      post_actions::table
        .filter(post_actions::post_id.eq(post_read_form.post_id))
        .filter(post_actions::person_id.eq(post_read_form.person_id)),
    )
    .set_null(post_actions::read)
    .get_result(conn)
    .await
  }

  pub async fn mark_many_as_read(
    pool: &mut DbPool<'_>,
    post_ids: &[PostId],
    person_id: PersonId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;

    let forms = post_ids
      .iter()
      .map(|post_id| (PostReadForm::new(*post_id, person_id)))
      .collect::<Vec<_>>();

    insert_into(post_actions::table)
      .values(forms)
      .on_conflict((post_actions::person_id, post_actions::post_id))
      .do_update()
      .set(post_actions::read.eq(now().nullable()))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntMarkPostAsRead)
  }
}

impl PostHide {
  pub async fn hide(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    person_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;

    let form = &PostHideForm::new(post_id, person_id);
    insert_into(post_actions::table)
      .values(form)
      .on_conflict((post_actions::person_id, post_actions::post_id))
      .do_update()
      .set(form)
      .execute(conn)
      .await
  }

  pub async fn unhide(
    pool: &mut DbPool<'_>,
    post_id_: PostId,
    person_id_: PersonId,
  ) -> Result<uplete::Count, Error> {
    let conn = &mut get_conn(pool).await?;

    uplete::new(
      post_actions::table
        .filter(post_actions::post_id.eq(post_id_))
        .filter(post_actions::person_id.eq(person_id_)),
    )
    .set_null(post_actions::hidden)
    .get_result(conn)
    .await
  }
}

impl PostActionsCursor {
  pub async fn read(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    Ok(if let Some(person_id) = person_id {
      post_actions::table
        .find((person_id, post_id))
        .select(Self::as_select())
        .first(conn)
        .await
        .optional()?
        .unwrap_or_default()
    } else {
      Default::default()
    })
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
    traits::{Crud, Likeable, Saveable},
    utils::{build_db_pool_for_tests, uplete},
  };
  use chrono::DateTime;
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;
  use url::Url;

  #[tokio::test]
  #[serial]
  async fn test_crud() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "jim");

    let inserted_person = Person::create(pool, &new_person).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "test community_3".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );

    let inserted_community = Community::create(pool, &new_community).await?;

    let new_post = PostInsertForm::new(
      "A test post".into(),
      inserted_person.id,
      inserted_community.id,
    );
    let inserted_post = Post::create(pool, &new_post).await?;

    let new_post2 = PostInsertForm::new(
      "A test post 2".into(),
      inserted_person.id,
      inserted_community.id,
    );
    let inserted_post2 = Post::create(pool, &new_post2).await?;

    let new_scheduled_post = PostInsertForm {
      scheduled_publish_time: Some(DateTime::from_timestamp_nanos(i64::MAX)),
      ..PostInsertForm::new("beans".into(), inserted_person.id, inserted_community.id)
    };
    let inserted_scheduled_post = Post::create(pool, &new_scheduled_post).await?;

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
      ap_id: Url::parse(&format!("https://lemmy-alpha/post/{}", inserted_post.id))?.into(),
      local: true,
      language_id: Default::default(),
      featured_community: false,
      featured_local: false,
      url_content_type: None,
      scheduled_publish_time: None,
    };

    // Post Like
    let post_like_form = PostLikeForm::new(inserted_post.id, inserted_person.id, 1);

    let inserted_post_like = PostLike::like(pool, &post_like_form).await?;

    let expected_post_like = PostLike {
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      published: inserted_post_like.published,
      score: 1,
    };

    // Post Save
    let post_saved_form = PostSavedForm::new(inserted_post.id, inserted_person.id);

    let inserted_post_saved = PostSaved::save(pool, &post_saved_form).await?;

    let expected_post_saved = PostSaved {
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      published: inserted_post_saved.published,
    };

    // Mark 2 posts as read
    let post_read_form_1 = PostReadForm::new(inserted_post.id, inserted_person.id);
    PostRead::mark_as_read(pool, &post_read_form_1).await?;
    let post_read_form_2 = PostReadForm::new(inserted_post2.id, inserted_person.id);
    PostRead::mark_as_read(pool, &post_read_form_2).await?;

    let read_post = Post::read(pool, inserted_post.id).await?;

    let new_post_update = PostUpdateForm {
      name: Some("A test post".into()),
      ..Default::default()
    };
    let updated_post = Post::update(pool, inserted_post.id, &new_post_update).await?;

    // Scheduled post count
    let scheduled_post_count = Post::user_scheduled_post_count(inserted_person.id, pool).await?;
    assert_eq!(1, scheduled_post_count);

    let like_removed = PostLike::remove(pool, inserted_person.id, inserted_post.id).await?;
    assert_eq!(uplete::Count::only_updated(1), like_removed);
    let saved_removed = PostSaved::unsave(pool, &post_saved_form).await?;
    assert_eq!(uplete::Count::only_updated(1), saved_removed);

    let read_remove_form_1 = PostReadForm::new(inserted_post.id, inserted_person.id);
    let read_removed_1 = PostRead::mark_as_unread(pool, &read_remove_form_1).await?;
    assert_eq!(uplete::Count::only_deleted(1), read_removed_1);

    let read_remove_form_2 = PostReadForm::new(inserted_post2.id, inserted_person.id);
    let read_removed_2 = PostRead::mark_as_unread(pool, &read_remove_form_2).await?;
    assert_eq!(uplete::Count::only_deleted(1), read_removed_2);

    let num_deleted = Post::delete(pool, inserted_post.id).await?
      + Post::delete(pool, inserted_post2.id).await?
      + Post::delete(pool, inserted_scheduled_post.id).await?;

    assert_eq!(3, num_deleted);
    Community::delete(pool, inserted_community.id).await?;
    Person::delete(pool, inserted_person.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    assert_eq!(expected_post, read_post);
    assert_eq!(expected_post, inserted_post);
    assert_eq!(expected_post, updated_post);
    assert_eq!(expected_post_like, inserted_post_like);
    assert_eq!(expected_post_saved, inserted_post_saved);

    Ok(())
  }
}
