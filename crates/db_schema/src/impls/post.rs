use crate::{
  newtypes::{CommunityId, DbUrl, InstanceId, PersonId, PostId},
  source::post::{
    Post,
    PostActions,
    PostActionsCursor,
    PostHideForm,
    PostInsertForm,
    PostLikeForm,
    PostReadCommentsForm,
    PostReadForm,
    PostSavedForm,
    PostSubscribeForm,
    PostUpdateForm,
  },
  traits::{Crud, Hideable, Likeable, ReadComments, Readable, Saveable},
  utils::{
    functions::{coalesce, hot_rank, scaled_rank},
    get_conn,
    now,
    uplete,
    DbConn,
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
  dsl::{count, insert_into, not, update},
  expression::SelectableHelper,
  result::Error,
  BoolExpressionMethods,
  DecoratableTarget,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  OptionalExtension,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{community, person, post, post_actions};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};

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
    for_instance_id: Option<InstanceId>,
    removed: bool,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;

    // Diesel can't update from join unfortunately, so you'll need to loop over these
    let community_join = community::table.on(post::community_id.eq(community::id));
    let mut posts_query = post::table
      .inner_join(community_join)
      .filter(post::creator_id.eq(for_creator_id))
      .into_boxed();

    if let Some(for_community_id) = for_community_id {
      posts_query = posts_query.filter(post::community_id.eq(for_community_id));
    }

    if let Some(for_instance_id) = for_instance_id {
      posts_query = posts_query.filter(community::instance_id.eq(for_instance_id));
    }

    let post_ids = posts_query.select(post::id).load::<PostId>(conn).await?;

    update(post::table)
      .filter(post::id.eq_any(post_ids.clone()))
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

  pub async fn delete_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: Url,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let object_id: DbUrl = object_id.into();

    diesel::update(post::table.filter(post::ap_id.eq(object_id)))
      .set(post::deleted.eq(true))
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

  pub async fn update_ranks(pool: &mut DbPool<'_>, post_id: PostId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    // Diesel can't update based on a join, which is necessary for the scaled_rank
    // https://github.com/diesel-rs/diesel/issues/1478
    // Just select the metrics we need manually, for now, since its a single post anyway

    let interactions_month = community::table
      .select(community::interactions_month)
      .inner_join(post::table.on(community::id.eq(post::community_id)))
      .filter(post::id.eq(post_id))
      .first::<i64>(conn)
      .await?;

    diesel::update(post::table.find(post_id))
      .set((
        post::hot_rank.eq(hot_rank(post::score, post::published)),
        post::hot_rank_active.eq(hot_rank(post::score, post::newest_comment_time_necro)),
        post::scaled_rank.eq(scaled_rank(
          post::score,
          post::published,
          interactions_month,
        )),
      ))
      .get_result::<Self>(conn)
      .await
  }
  pub fn local_url(&self, settings: &Settings) -> LemmyResult<DbUrl> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/post/{}", self.id))?.into())
  }

  /// The comment was created locally and sent back, indicating that the community accepted it
  pub async fn set_not_pending(&self, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    if self.local && self.federation_pending {
      let form = PostUpdateForm {
        federation_pending: Some(false),
        ..Default::default()
      };
      Post::update(pool, self.id, &form).await?;
    }
    Ok(())
  }
}

impl Likeable for PostActions {
  type Form = PostLikeForm;
  type IdType = PostId;

  async fn like(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_actions::table)
      .values(form)
      .on_conflict((post_actions::post_id, post_actions::person_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntLikePost)
  }
  async fn remove_like(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    post_id: Self::IdType,
  ) -> LemmyResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(post_actions::table.find((person_id, post_id)))
      .set_null(post_actions::like_score)
      .set_null(post_actions::liked)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntLikePost)
  }
}

impl Saveable for PostActions {
  type Form = PostSavedForm;
  async fn save(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_actions::table)
      .values(form)
      .on_conflict((post_actions::post_id, post_actions::person_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntSavePost)
  }
  async fn unsave(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(post_actions::table.find((form.person_id, form.post_id)))
      .set_null(post_actions::saved)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntSavePost)
  }
}

impl Readable for PostActions {
  type Form = PostReadForm;

  async fn mark_as_read(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<usize> {
    Self::mark_many_as_read(pool, &[form.clone()]).await
  }

  async fn mark_as_unread(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;

    uplete::new(
      post_actions::table
        .filter(post_actions::post_id.eq(form.post_id))
        .filter(post_actions::person_id.eq(form.person_id)),
    )
    .set_null(post_actions::read)
    .get_result(conn)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntMarkPostAsRead)
  }

  async fn mark_many_as_read(pool: &mut DbPool<'_>, forms: &[Self::Form]) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;

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

impl Hideable for PostActions {
  type Form = PostHideForm;
  async fn hide(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(post_actions::table)
      .values(form)
      .on_conflict((post_actions::person_id, post_actions::post_id))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntHidePost)
  }

  async fn unhide(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;

    uplete::new(
      post_actions::table
        .filter(post_actions::post_id.eq(form.post_id))
        .filter(post_actions::person_id.eq(form.person_id)),
    )
    .set_null(post_actions::hidden)
    .get_result(conn)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntHidePost)
  }
}

impl ReadComments for PostActions {
  type Form = PostReadCommentsForm;
  type IdType = PostId;

  async fn update_read_comments(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(post_actions::table)
      .values(form)
      .on_conflict((post_actions::person_id, post_actions::post_id))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdateReadComments)
  }

  async fn remove_read_comments(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    post_id: Self::IdType,
  ) -> LemmyResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;

    uplete::new(
      post_actions::table
        .filter(post_actions::post_id.eq(post_id))
        .filter(post_actions::person_id.eq(person_id)),
    )
    .set_null(post_actions::read_comments_amount)
    .set_null(post_actions::read_comments)
    .get_result(conn)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateReadComments)
  }
}

impl PostActions {
  pub fn build_many_read_forms(post_ids: &[PostId], person_id: PersonId) -> Vec<PostReadForm> {
    post_ids
      .iter()
      .map(|post_id| (PostReadForm::new(*post_id, person_id)))
      .collect::<Vec<_>>()
  }

  pub async fn subscribe(pool: &mut DbPool<'_>, form: &PostSubscribeForm) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_actions::table)
      .values(form)
      .on_conflict((post_actions::person_id, post_actions::post_id))
      .do_update()
      .set(form)
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntSubscribePost)?;
    Ok(())
  }

  pub async fn unsubscribe(
    pool: &mut DbPool<'_>,
    form: &PostSubscribeForm,
  ) -> LemmyResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;

    uplete::new(
      post_actions::table
        .filter(post_actions::post_id.eq(form.post_id))
        .filter(post_actions::person_id.eq(form.person_id)),
    )
    .set_null(post_actions::subscribed)
    .get_result(conn)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntSubscribePost)
  }
}

impl PostActionsCursor {
  pub async fn read(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    Self::read_conn(conn, post_id, person_id).await
  }

  pub async fn read_conn(
    conn: &mut DbConn<'_>,
    post_id: PostId,
    person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
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
      comment::{Comment, CommentInsertForm, CommentUpdateForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{
        Post,
        PostActions,
        PostInsertForm,
        PostLikeForm,
        PostReadForm,
        PostSavedForm,
        PostUpdateForm,
      },
    },
    traits::{Crud, Likeable, Readable, Saveable},
    utils::{build_db_pool_for_tests, uplete, RANK_DEFAULT},
  };
  use chrono::DateTime;
  use diesel::result::Error;
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
      comments: 0,
      controversy_rank: 0.0,
      downvotes: 0,
      upvotes: 1,
      score: 1,
      hot_rank: RANK_DEFAULT,
      hot_rank_active: RANK_DEFAULT,
      newest_comment_time: inserted_post.published,
      newest_comment_time_necro: inserted_post.published,
      report_count: 0,
      scaled_rank: RANK_DEFAULT,
      unresolved_report_count: 0,
      federation_pending: false,
    };

    // Post Like
    let post_like_form = PostLikeForm::new(inserted_post.id, inserted_person.id, 1);

    let inserted_post_like = PostActions::like(pool, &post_like_form).await?;
    assert_eq!(Some(1), inserted_post_like.like_score);

    // Post Save
    let post_saved_form = PostSavedForm::new(inserted_post.id, inserted_person.id);

    let inserted_post_saved = PostActions::save(pool, &post_saved_form).await?;
    assert!(inserted_post_saved.saved.is_some());

    // Mark 2 posts as read
    let post_read_form_1 = PostReadForm::new(inserted_post.id, inserted_person.id);
    PostActions::mark_as_read(pool, &post_read_form_1).await?;
    let post_read_form_2 = PostReadForm::new(inserted_post2.id, inserted_person.id);
    PostActions::mark_as_read(pool, &post_read_form_2).await?;

    let read_post = Post::read(pool, inserted_post.id).await?;

    let new_post_update = PostUpdateForm {
      name: Some("A test post".into()),
      ..Default::default()
    };
    let updated_post = Post::update(pool, inserted_post.id, &new_post_update).await?;

    // Scheduled post count
    let scheduled_post_count = Post::user_scheduled_post_count(inserted_person.id, pool).await?;
    assert_eq!(1, scheduled_post_count);

    let like_removed = PostActions::remove_like(pool, inserted_person.id, inserted_post.id).await?;
    assert_eq!(uplete::Count::only_updated(1), like_removed);
    let saved_removed = PostActions::unsave(pool, &post_saved_form).await?;
    assert_eq!(uplete::Count::only_updated(1), saved_removed);

    let read_remove_form_1 = PostReadForm::new(inserted_post.id, inserted_person.id);
    let read_removed_1 = PostActions::mark_as_unread(pool, &read_remove_form_1).await?;
    assert_eq!(uplete::Count::only_deleted(1), read_removed_1);

    let read_remove_form_2 = PostReadForm::new(inserted_post2.id, inserted_person.id);
    let read_removed_2 = PostActions::mark_as_unread(pool, &read_remove_form_2).await?;
    assert_eq!(uplete::Count::only_deleted(1), read_removed_2);

    let num_deleted = Post::delete(pool, inserted_post.id).await?
      + Post::delete(pool, inserted_post2.id).await?
      + Post::delete(pool, inserted_scheduled_post.id).await?;

    assert_eq!(3, num_deleted);
    Community::delete(pool, inserted_community.id).await?;
    Person::delete(pool, inserted_person.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    assert_eq!(expected_post, read_post);
    assert_eq!(expected_post, updated_post);

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_aggregates() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "thommy_community_agg");

    let inserted_person = Person::create(pool, &new_person).await?;

    let another_person = PersonInsertForm::test_form(inserted_instance.id, "jerry_community_agg");

    let another_inserted_person = Person::create(pool, &another_person).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "TIL_community_agg".into(),
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

    let comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );
    let inserted_comment = Comment::create(pool, &comment_form, None).await?;

    let child_comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );
    let inserted_child_comment =
      Comment::create(pool, &child_comment_form, Some(&inserted_comment.path)).await?;

    let post_like = PostLikeForm::new(inserted_post.id, inserted_person.id, 1);

    PostActions::like(pool, &post_like).await?;

    let post_aggs_before_delete = Post::read(pool, inserted_post.id).await?;

    assert_eq!(2, post_aggs_before_delete.comments);
    assert_eq!(1, post_aggs_before_delete.score);
    assert_eq!(1, post_aggs_before_delete.upvotes);
    assert_eq!(0, post_aggs_before_delete.downvotes);

    // Add a post dislike from the other person
    let post_dislike = PostLikeForm::new(inserted_post.id, another_inserted_person.id, -1);

    PostActions::like(pool, &post_dislike).await?;

    let post_aggs_after_dislike = Post::read(pool, inserted_post.id).await?;

    assert_eq!(2, post_aggs_after_dislike.comments);
    assert_eq!(0, post_aggs_after_dislike.score);
    assert_eq!(1, post_aggs_after_dislike.upvotes);
    assert_eq!(1, post_aggs_after_dislike.downvotes);

    // Remove the comments
    Comment::delete(pool, inserted_comment.id).await?;
    Comment::delete(pool, inserted_child_comment.id).await?;
    let after_comment_delete = Post::read(pool, inserted_post.id).await?;
    assert_eq!(0, after_comment_delete.comments);
    assert_eq!(0, after_comment_delete.score);
    assert_eq!(1, after_comment_delete.upvotes);
    assert_eq!(1, after_comment_delete.downvotes);

    // Remove the first post like
    PostActions::remove_like(pool, inserted_person.id, inserted_post.id).await?;
    let after_like_remove = Post::read(pool, inserted_post.id).await?;
    assert_eq!(0, after_like_remove.comments);
    assert_eq!(-1, after_like_remove.score);
    assert_eq!(0, after_like_remove.upvotes);
    assert_eq!(1, after_like_remove.downvotes);

    // This should delete all the associated rows, and fire triggers
    Person::delete(pool, another_inserted_person.id).await?;
    let person_num_deleted = Person::delete(pool, inserted_person.id).await?;
    assert_eq!(1, person_num_deleted);

    // Delete the community
    let community_num_deleted = Community::delete(pool, inserted_community.id).await?;
    assert_eq!(1, community_num_deleted);

    // Should be none found, since the creator was deleted
    let after_delete = Post::read(pool, inserted_post.id).await;
    assert!(after_delete.is_err());

    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_aggregates_soft_delete() -> Result<(), Error> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "thommy_community_agg");

    let inserted_person = Person::create(pool, &new_person).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "TIL_community_agg".into(),
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

    let comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );

    let inserted_comment = Comment::create(pool, &comment_form, None).await?;

    let post_aggregates_before = Post::read(pool, inserted_post.id).await?;
    assert_eq!(1, post_aggregates_before.comments);

    Comment::update(
      pool,
      inserted_comment.id,
      &CommentUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let post_aggregates_after_remove = Post::read(pool, inserted_post.id).await?;
    assert_eq!(0, post_aggregates_after_remove.comments);

    Comment::update(
      pool,
      inserted_comment.id,
      &CommentUpdateForm {
        removed: Some(false),
        ..Default::default()
      },
    )
    .await?;

    Comment::update(
      pool,
      inserted_comment.id,
      &CommentUpdateForm {
        deleted: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let post_aggregates_after_delete = Post::read(pool, inserted_post.id).await?;
    assert_eq!(0, post_aggregates_after_delete.comments);

    Comment::update(
      pool,
      inserted_comment.id,
      &CommentUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let post_aggregates_after_delete_remove = Post::read(pool, inserted_post.id).await?;
    assert_eq!(0, post_aggregates_after_delete_remove.comments);

    Comment::delete(pool, inserted_comment.id).await?;
    Post::delete(pool, inserted_post.id).await?;
    Person::delete(pool, inserted_person.id).await?;
    Community::delete(pool, inserted_community.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }
}
