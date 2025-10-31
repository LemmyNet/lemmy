use crate::{
  newtypes::{CommunityId, DbUrl, InstanceId, PaginationCursor, PersonId, PostId},
  source::post::{
    Post,
    PostActions,
    PostHideForm,
    PostInsertForm,
    PostLikeForm,
    PostReadCommentsForm,
    PostReadForm,
    PostSavedForm,
    PostUpdateForm,
  },
  traits::{Crud, Likeable, Saveable},
  utils::{
    functions::{coalesce, hot_rank, scaled_rank},
    get_conn,
    now,
    DbPool,
    DELETED_REPLACEMENT_TEXT,
    FETCH_LIMIT_MAX,
    SITEMAP_DAYS,
    SITEMAP_LIMIT,
  },
};
use chrono::{DateTime, Utc};
use diesel::{
  dsl::{count, insert_into, not, update},
  expression::SelectableHelper,
  BoolExpressionMethods,
  DecoratableTarget,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  OptionalExtension,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use diesel_uplete::{uplete, UpleteCount};
use lemmy_db_schema_file::{
  enums::PostNotificationsMode,
  schema::{community, local_user, person, post, post_actions},
};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use url::Url;

impl Crud for Post {
  type InsertForm = PostInsertForm;
  type UpdateForm = PostUpdateForm;
  type IdType = PostId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    new_post: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(post::table.find(post_id))
      .set(new_post)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl Post {
  pub async fn read(pool: &mut DbPool<'_>, id: PostId) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    post::table
      .find(id)
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn insert_apub(
    pool: &mut DbPool<'_>,
    timestamp: DateTime<Utc>,
    form: &PostInsertForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post::table)
      .values(form)
      .on_conflict(post::ap_id)
      .filter_target(coalesce(post::updated_at, post::published_at).lt(timestamp))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  pub async fn list_featured_for_community(
    pool: &mut DbPool<'_>,
    the_community_id: CommunityId,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    post::table
      .filter(post::community_id.eq(the_community_id))
      .filter(post::deleted.eq(false))
      .filter(post::removed.eq(false))
      .filter(post::featured_community.eq(true))
      .then_order_by(post::published_at.desc())
      .limit(FETCH_LIMIT_MAX.try_into()?)
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn list_for_sitemap(
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Vec<(DbUrl, chrono::DateTime<Utc>)>> {
    let conn = &mut get_conn(pool).await?;
    post::table
      .select((post::ap_id, coalesce(post::updated_at, post::published_at)))
      .filter(post::local.eq(true))
      .filter(post::deleted.eq(false))
      .filter(post::removed.eq(false))
      .filter(post::published_at.ge(Utc::now().naive_utc() - SITEMAP_DAYS))
      .order(post::published_at.desc())
      .limit(SITEMAP_LIMIT)
      .load::<(DbUrl, chrono::DateTime<Utc>)>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn permadelete_for_creator(
    pool: &mut DbPool<'_>,
    for_creator_id: PersonId,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    diesel::update(post::table.filter(post::creator_id.eq(for_creator_id)))
      .set((
        post::name.eq(DELETED_REPLACEMENT_TEXT),
        post::url.eq(Option::<&str>::None),
        post::body.eq(DELETED_REPLACEMENT_TEXT),
        post::deleted.eq(true),
        post::updated_at.eq(Utc::now()),
      ))
      .get_results::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  async fn creator_post_ids_in_community(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    community_id: CommunityId,
  ) -> LemmyResult<Vec<PostId>> {
    let conn = &mut get_conn(pool).await?;

    post::table
      .filter(post::creator_id.eq(creator_id))
      .filter(post::community_id.eq(community_id))
      .select(post::id)
      .load::<PostId>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Diesel can't update from join unfortunately, so you sometimes need to fetch a list of post_ids
  /// for a creator.
  async fn creator_post_ids_in_instance(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    instance_id: InstanceId,
  ) -> LemmyResult<Vec<PostId>> {
    let conn = &mut get_conn(pool).await?;

    post::table
      .inner_join(community::table)
      .filter(post::creator_id.eq(creator_id))
      .filter(community::instance_id.eq(instance_id))
      .select(post::id)
      .load::<PostId>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn update_removed_for_creator_and_community(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    community_id: CommunityId,
    removed: bool,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    update(post::table)
      .filter(post::creator_id.eq(creator_id))
      .filter(post::community_id.eq(community_id))
      .set((post::removed.eq(removed), post::updated_at.eq(Utc::now())))
      .get_results::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  pub async fn update_removed_for_creator_and_instance(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    instance_id: InstanceId,
    removed: bool,
  ) -> LemmyResult<Vec<Self>> {
    let post_ids = Self::creator_post_ids_in_instance(pool, creator_id, instance_id).await?;

    let conn = &mut get_conn(pool).await?;

    update(post::table)
      .filter(post::id.eq_any(post_ids.clone()))
      .set((post::removed.eq(removed), post::updated_at.eq(Utc::now())))
      .get_results(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  pub async fn update_removed_for_creator(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    removed: bool,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    update(post::table)
      .filter(post::creator_id.eq(creator_id))
      .set((post::removed.eq(removed), post::updated_at.eq(Utc::now())))
      .get_results(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  pub fn is_post_creator(person_id: PersonId, post_creator_id: PersonId) -> bool {
    person_id == post_creator_id
  }

  pub async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: DbUrl,
  ) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    post::table
      .filter(post::ap_id.eq(object_id))
      .filter(post::scheduled_publish_time_at.is_null())
      .first(conn)
      .await
      .optional()
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn delete_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: Url,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    let object_id: DbUrl = object_id.into();

    diesel::update(post::table.filter(post::ap_id.eq(object_id)))
      .set(post::deleted.eq(true))
      .get_results::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  pub async fn user_scheduled_post_count(
    person_id: PersonId,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<i64> {
    let conn = &mut get_conn(pool).await?;

    post::table
      .inner_join(person::table)
      .inner_join(community::table)
      // find all posts which have scheduled_publish_time that is in the  future
      .filter(post::scheduled_publish_time_at.is_not_null())
      .filter(coalesce(post::scheduled_publish_time_at, now()).gt(now()))
      // make sure the post and community are still around
      .filter(not(post::deleted.or(post::removed)))
      .filter(not(community::removed.or(community::deleted)))
      // only posts by specified user
      .filter(post::creator_id.eq(person_id))
      .select(count(post::id))
      .first::<i64>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn update_ranks(pool: &mut DbPool<'_>, post_id: PostId) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    // Diesel can't update based on a join, which is necessary for the scaled_rank
    // https://github.com/diesel-rs/diesel/issues/1478
    // Just select the metrics we need manually, for now, since its a single post anyway

    let interactions_month = community::table
      .select(community::interactions_month)
      .inner_join(post::table.on(community::id.eq(post::community_id)))
      .filter(post::id.eq(post_id))
      .first::<i32>(conn)
      .await?;

    diesel::update(post::table.find(post_id))
      .set((
        post::hot_rank.eq(hot_rank(post::score, post::published_at)),
        post::hot_rank_active.eq(hot_rank(
          post::score,
          coalesce(post::newest_comment_time_necro_at, post::published_at),
        )),
        post::scaled_rank.eq(scaled_rank(
          post::score,
          post::published_at,
          interactions_month,
        )),
      ))
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
  pub fn local_url(&self, settings: &Settings) -> LemmyResult<Url> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/post/{}", self.id))?)
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
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn remove_like(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    post_id: Self::IdType,
  ) -> LemmyResult<UpleteCount> {
    let conn = &mut get_conn(pool).await?;
    uplete(post_actions::table.find((person_id, post_id)))
      .set_null(post_actions::vote_is_upvote)
      .set_null(post_actions::voted_at)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  async fn remove_all_likes(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> LemmyResult<UpleteCount> {
    let conn = &mut get_conn(pool).await?;

    uplete(post_actions::table.filter(post_actions::person_id.eq(person_id)))
      .set_null(post_actions::vote_is_upvote)
      .set_null(post_actions::voted_at)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  async fn remove_likes_in_community(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    community_id: CommunityId,
  ) -> LemmyResult<UpleteCount> {
    let post_ids = Post::creator_post_ids_in_community(pool, person_id, community_id).await?;

    let conn = &mut get_conn(pool).await?;

    uplete(post_actions::table.filter(post_actions::post_id.eq_any(post_ids.clone())))
      .set_null(post_actions::vote_is_upvote)
      .set_null(post_actions::voted_at)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
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
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
  async fn unsave(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<UpleteCount> {
    let conn = &mut get_conn(pool).await?;
    uplete(post_actions::table.find((form.person_id, form.post_id)))
      .set_null(post_actions::saved_at)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl PostActions {
  pub async fn mark_as_unread(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    post_ids: &[PostId],
  ) -> LemmyResult<UpleteCount> {
    let conn = &mut get_conn(pool).await?;

    let post_ids: Vec<_> = post_ids.to_vec();
    uplete(
      post_actions::table
        .filter(post_actions::post_id.eq_any(post_ids))
        .filter(post_actions::person_id.eq(person_id)),
    )
    .set_null(post_actions::read_at)
    .get_result(conn)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  pub async fn mark_as_read(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    post_ids: &[PostId],
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;

    let forms: Vec<_> = post_ids
      .iter()
      .map(|post_id| PostReadForm::new(*post_id, person_id))
      .collect();

    insert_into(post_actions::table)
      .values(forms)
      .on_conflict((post_actions::person_id, post_actions::post_id))
      .do_update()
      .set(post_actions::read_at.eq(now().nullable()))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl PostActions {
  pub async fn hide(pool: &mut DbPool<'_>, form: &PostHideForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(post_actions::table)
      .values(form)
      .on_conflict((post_actions::person_id, post_actions::post_id))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  pub async fn unhide(pool: &mut DbPool<'_>, form: &PostHideForm) -> LemmyResult<UpleteCount> {
    let conn = &mut get_conn(pool).await?;

    uplete(
      post_actions::table
        .filter(post_actions::post_id.eq(form.post_id))
        .filter(post_actions::person_id.eq(form.person_id)),
    )
    .set_null(post_actions::hidden_at)
    .get_result(conn)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl PostActions {
  pub async fn update_read_comments(
    pool: &mut DbPool<'_>,
    form: &PostReadCommentsForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    insert_into(post_actions::table)
      .values(form)
      .on_conflict((post_actions::person_id, post_actions::post_id))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl PostActions {
  pub async fn read(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    person_id: PersonId,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    post_actions::table
      .find((person_id, post_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<PostActions> {
    let [(_, person_id), (_, post_id)] = cursor.prefixes_and_ids()?;
    Self::read(pool, PostId(post_id), PersonId(person_id)).await
  }

  pub async fn update_notification_state(
    post_id: PostId,
    person_id: PersonId,
    new_state: PostNotificationsMode,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    let form = (
      post_actions::person_id.eq(person_id),
      post_actions::post_id.eq(post_id),
      post_actions::notifications.eq(new_state),
    );

    insert_into(post_actions::table)
      .values(form.clone())
      .on_conflict((post_actions::person_id, post_actions::post_id))
      .do_update()
      .set(form)
      .execute(conn)
      .await?;
    Ok(())
  }

  pub async fn list_subscribers(
    post_id: PostId,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Vec<PersonId>> {
    let conn = &mut get_conn(pool).await?;

    post_actions::table
      .inner_join(local_user::table.on(post_actions::person_id.eq(local_user::person_id)))
      .filter(post_actions::post_id.eq(post_id))
      .filter(post_actions::notifications.eq(PostNotificationsMode::AllComments))
      .select(local_user::person_id)
      .get_results(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
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
      post::{Post, PostActions, PostInsertForm, PostLikeForm, PostSavedForm, PostUpdateForm},
    },
    traits::{Crud, Likeable, Saveable},
    utils::{build_db_pool_for_tests, RANK_DEFAULT},
  };
  use chrono::DateTime;
  use diesel_uplete::UpleteCount;
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;
  use url::Url;

  #[tokio::test]
  #[serial]
  async fn test_crud() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld").await?;

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
      scheduled_publish_time_at: Some(DateTime::from_timestamp_nanos(i64::MAX)),
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
      published_at: inserted_post.published_at,
      removed: false,
      locked: false,
      nsfw: false,
      deleted: false,
      updated_at: None,
      embed_title: None,
      embed_description: None,
      embed_video_url: None,
      embed_video_width: None,
      embed_video_height: None,
      thumbnail_url: None,
      ap_id: Url::parse(&format!("https://lemmy-alpha/post/{}", inserted_post.id))?.into(),
      local: true,
      language_id: Default::default(),
      featured_community: false,
      featured_local: false,
      url_content_type: None,
      scheduled_publish_time_at: None,
      comments: 0,
      controversy_rank: 0.0,
      downvotes: 0,
      upvotes: 1,
      score: 1,
      hot_rank: RANK_DEFAULT,
      hot_rank_active: RANK_DEFAULT,
      newest_comment_time_at: None,
      newest_comment_time_necro_at: None,
      report_count: 0,
      scaled_rank: RANK_DEFAULT,
      unresolved_report_count: 0,
      federation_pending: false,
    };

    // Post Like
    let post_like_form = PostLikeForm::new(inserted_post.id, inserted_person.id, true);

    let inserted_post_like = PostActions::like(pool, &post_like_form).await?;
    assert_eq!(Some(true), inserted_post_like.vote_is_upvote);

    // Post Save
    let post_saved_form = PostSavedForm::new(inserted_post.id, inserted_person.id);

    let inserted_post_saved = PostActions::save(pool, &post_saved_form).await?;
    assert!(inserted_post_saved.saved_at.is_some());

    // Mark 2 posts as read
    PostActions::mark_as_read(pool, inserted_person.id, &[inserted_post.id]).await?;
    PostActions::mark_as_read(pool, inserted_person.id, &[inserted_post2.id]).await?;

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
    assert_eq!(UpleteCount::only_updated(1), like_removed);
    let saved_removed = PostActions::unsave(pool, &post_saved_form).await?;
    assert_eq!(UpleteCount::only_updated(1), saved_removed);

    let read_removed_1 =
      PostActions::mark_as_unread(pool, inserted_person.id, &[inserted_post.id]).await?;
    assert_eq!(UpleteCount::only_deleted(1), read_removed_1);

    let read_removed_2 =
      PostActions::mark_as_unread(pool, inserted_person.id, &[inserted_post2.id]).await?;
    assert_eq!(UpleteCount::only_deleted(1), read_removed_2);

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

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld").await?;

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

    let post_like = PostLikeForm::new(inserted_post.id, inserted_person.id, true);

    PostActions::like(pool, &post_like).await?;

    let post_aggs_before_delete = Post::read(pool, inserted_post.id).await?;

    assert_eq!(2, post_aggs_before_delete.comments);
    assert_eq!(1, post_aggs_before_delete.score);
    assert_eq!(1, post_aggs_before_delete.upvotes);
    assert_eq!(0, post_aggs_before_delete.downvotes);

    // Add a post dislike from the other person
    let post_dislike = PostLikeForm::new(inserted_post.id, another_inserted_person.id, false);

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
  async fn test_aggregates_soft_delete() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld").await?;

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
