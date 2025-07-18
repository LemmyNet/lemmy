use crate::{
  diesel::{DecoratableTarget, OptionalExtension},
  newtypes::{CommentId, CommunityId, DbUrl, InstanceId, PersonId},
  source::{
    comment::{
      Comment,
      CommentActions,
      CommentInsertForm,
      CommentLikeForm,
      CommentSavedForm,
      CommentUpdateForm,
    },
    history_status::{HistoryStatus, HistoryStatusUpdateForm},
  },
  traits::{Crud, Likeable, Saveable},
  utils::{
    functions::{coalesce, hot_rank},
    get_conn,
    validate_like,
    DbPool,
    DELETED_REPLACEMENT_TEXT,
  },
};
use chrono::{DateTime, Utc};
use diesel::{
  dsl::{count_star, delete, insert_into},
  expression::SelectableHelper,
  sql_query,
  sql_types::{BigInt, Integer},
  update,
  upsert::excluded,
  ExpressionMethods,
  JoinOnDsl,
  QueryDsl,
};
use diesel_async::{scoped_futures::ScopedFutureExt, RunQueryDsl};
use diesel_ltree::Ltree;
use diesel_uplete::{uplete, UpleteCount};
use lemmy_db_schema_file::schema::{
  comment,
  comment_actions,
  comment_aggregates,
  comment_like,
  community,
  post,
};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorExt2, LemmyErrorType, LemmyResult},
  settings::structs::Settings,
  DB_BATCH_SIZE,
};
use tracing::info;
use url::Url;

impl Comment {
  pub async fn permadelete_for_creator(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    diesel::update(comment::table.filter(comment::creator_id.eq(creator_id)))
      .set((
        comment::content.eq(DELETED_REPLACEMENT_TEXT),
        comment::deleted.eq(true),
        comment::updated_at.eq(Utc::now()),
      ))
      .get_results::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)
  }

  pub async fn update_removed_for_creator(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    removed: bool,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(comment::table.filter(comment::creator_id.eq(creator_id)))
      .set((
        comment::removed.eq(removed),
        comment::updated_at.eq(Utc::now()),
      ))
      .get_results::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)
  }

  /// Diesel can't update from join unfortunately, so you'll need to loop over these
  async fn creator_comment_ids_in_community(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    community_id: CommunityId,
  ) -> LemmyResult<Vec<CommentId>> {
    let conn = &mut get_conn(pool).await?;

    comment::table
      .inner_join(post::table)
      .filter(comment::creator_id.eq(creator_id))
      .filter(post::community_id.eq(community_id))
      .select(comment::id)
      .load::<CommentId>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Diesel can't update from join unfortunately, so you'll need to loop over these
  async fn creator_comment_ids_in_instance(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    instance_id: InstanceId,
  ) -> LemmyResult<Vec<CommentId>> {
    let conn = &mut get_conn(pool).await?;
    let community_join = community::table.on(post::community_id.eq(community::id));

    comment::table
      .inner_join(post::table)
      .inner_join(community_join)
      .filter(comment::creator_id.eq(creator_id))
      .filter(community::instance_id.eq(instance_id))
      .select(comment::id)
      .load::<CommentId>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn update_removed_for_creator_and_community(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    community_id: CommunityId,
    removed: bool,
  ) -> LemmyResult<Vec<CommentId>> {
    let comment_ids =
      Self::creator_comment_ids_in_community(pool, creator_id, community_id).await?;

    let conn = &mut get_conn(pool).await?;

    update(comment::table)
      .filter(comment::id.eq_any(comment_ids.clone()))
      .set((
        comment::removed.eq(removed),
        comment::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await?;

    Ok(comment_ids)
  }

  pub async fn update_removed_for_creator_and_instance(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    instance_id: InstanceId,
    removed: bool,
  ) -> LemmyResult<Vec<CommentId>> {
    let comment_ids = Self::creator_comment_ids_in_instance(pool, creator_id, instance_id).await?;
    let conn = &mut get_conn(pool).await?;

    update(comment::table)
      .filter(comment::id.eq_any(comment_ids.clone()))
      .set((
        comment::removed.eq(removed),
        comment::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await?;
    Ok(comment_ids)
  }

  pub async fn create(
    pool: &mut DbPool<'_>,
    comment_form: &CommentInsertForm,
    parent_path: Option<&Ltree>,
  ) -> LemmyResult<Comment> {
    Self::insert_apub(pool, None, comment_form, parent_path).await
  }

  pub async fn insert_apub(
    pool: &mut DbPool<'_>,
    timestamp: Option<DateTime<Utc>>,
    comment_form: &CommentInsertForm,
    parent_path: Option<&Ltree>,
  ) -> LemmyResult<Comment> {
    let conn = &mut get_conn(pool).await?;
    let comment_form = (comment_form, parent_path.map(|p| comment::path.eq(p)));

    if let Some(timestamp) = timestamp {
      insert_into(comment::table)
        .values(comment_form)
        .on_conflict(comment::ap_id)
        .filter_target(coalesce(comment::updated_at, comment::published_at).lt(timestamp))
        .do_update()
        .set(comment_form)
        .get_result::<Self>(conn)
        .await
        .with_lemmy_type(LemmyErrorType::CouldntCreateComment)
    } else {
      insert_into(comment::table)
        .values(comment_form)
        .get_result::<Self>(conn)
        .await
        .with_lemmy_type(LemmyErrorType::CouldntCreateComment)
    }
  }

  pub async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: Url,
  ) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    let object_id: DbUrl = object_id.into();
    comment::table
      .filter(comment::ap_id.eq(object_id))
      .first(conn)
      .await
      .optional()
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub fn parent_comment_id(&self) -> Option<CommentId> {
    let mut ltree_split: Vec<&str> = self.path.0.split('.').collect();
    ltree_split.remove(0); // The first is always 0
    if ltree_split.len() > 1 {
      let parent_comment_id = ltree_split.get(ltree_split.len() - 2);
      parent_comment_id.and_then(|p| p.parse::<i32>().map(CommentId).ok())
    } else {
      None
    }
  }
  pub async fn update_hot_rank(pool: &mut DbPool<'_>, comment_id: CommentId) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    diesel::update(comment::table.find(comment_id))
      .set(comment::hot_rank.eq(hot_rank(comment::score, comment::published_at)))
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)
  }
  pub fn local_url(&self, settings: &Settings) -> LemmyResult<Url> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/comment/{}", self.id))?)
  }

  /// The comment was created locally and sent back, indicating that the community accepted it
  pub async fn set_not_pending(&self, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    if self.local && self.federation_pending {
      let form = CommentUpdateForm {
        federation_pending: Some(false),
        ..Default::default()
      };
      Comment::update(pool, self.id, &form).await?;
    }
    Ok(())
  }

  pub async fn fill_aggregates_history(pool: &mut DbPool<'_>) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;

    info!("Filling comment_aggregates history into comment...");

    // Get the total count of comment_aggregates rows, to show progress
    let comment_aggregates_count = comment_aggregates::table
      .select(count_star())
      .first::<i64>(conn)
      .await?;

    let mut processed_rows = 0;

    while processed_rows < comment_aggregates_count {
      let rows_updated = conn
        .run_transaction(|conn| {
          async move {
            // Diesel can't do 'update X from Y', nor updates from joins, so you need to do custom
            // sql. I also tried individual row sets, and it was too slow.
            let updated_rows = sql_query(
              r#"
              WITH ca AS (SELECT *
                FROM comment_aggregates ca
                ORDER BY ca.comment_id desc
                LIMIT $1)
              UPDATE comment c
                SET
                  score = ca.score,
                  upvotes = ca.upvotes,
                  downvotes = ca.downvotes,
                  child_count = ca.child_count,
                  hot_rank = ca.hot_rank,
                  controversy_rank = ca.controversy_rank,
                  report_count = ca.report_count,
                  unresolved_report_count = ca.unresolved_report_count
                FROM ca WHERE c.id = ca.comment_id
                RETURNING c.id;
            "#,
            )
            .bind::<BigInt, _>(DB_BATCH_SIZE)
            .get_results::<AggregatesUpdateResult>(conn)
            .await?;

            // When this is None, the scanning is complete
            let last_scanned_id = updated_rows.last().map(|f| f.id);

            if let Some(last_scanned_id) = last_scanned_id {
              // Update the history status
              let history_form = HistoryStatusUpdateForm {
                last_scanned_timestamp: None,
                last_scanned_id: Some(Some(last_scanned_id.0)),
              };
              HistoryStatus::update_conn(
                conn,
                ("comment_aggregates".into(), "comment".into()),
                &history_form,
              )
              .await?;

              // Delete those rows from comment_aggregates
              delete(
                comment_aggregates::table
                  .filter(comment_aggregates::comment_id.gt(last_scanned_id)),
              )
              .execute(conn)
              .await?;
            }

            Ok(updated_rows.len())
          }
          .scope_boxed()
        })
        .await?;

      processed_rows += i64::try_from(rows_updated)?;
      let pct_complete = processed_rows * 100 / comment_aggregates_count;
      info!(
        "comment_aggregates -> comment: {processed_rows} / {comment_aggregates_count} , {pct_complete}% complete"
      );
    }

    info!("Finished filling comment_aggregates history into comment.");
    Ok(())
  }
}

/// Used for a custom update query
#[derive(QueryableByName)]
struct AggregatesUpdateResult {
  #[diesel(sql_type = Integer)]
  id: CommentId,
}

impl Crud for Comment {
  type InsertForm = CommentInsertForm;
  type UpdateForm = CommentUpdateForm;
  type IdType = CommentId;

  /// Use [[Comment::create]]
  async fn create(pool: &mut DbPool<'_>, comment_form: &Self::InsertForm) -> LemmyResult<Self> {
    debug_assert!(false);
    Comment::create(pool, comment_form, None).await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
    comment_form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(comment::table.find(comment_id))
      .set(comment_form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)
  }
}

impl Likeable for CommentActions {
  type Form = CommentLikeForm;
  type IdType = CommentId;

  async fn like(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    validate_like(form.like_score).with_lemmy_type(LemmyErrorType::CouldntLikeComment)?;

    insert_into(comment_actions::table)
      .values(form)
      .on_conflict((comment_actions::comment_id, comment_actions::person_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntLikeComment)
  }
  async fn remove_like(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    comment_id: Self::IdType,
  ) -> LemmyResult<UpleteCount> {
    let conn = &mut get_conn(pool).await?;
    uplete(comment_actions::table.find((person_id, comment_id)))
      .set_null(comment_actions::like_score)
      .set_null(comment_actions::liked_at)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntLikeComment)
  }

  async fn remove_all_likes(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
  ) -> LemmyResult<UpleteCount> {
    let conn = &mut get_conn(pool).await?;

    uplete(comment_actions::table.filter(comment_actions::person_id.eq(creator_id)))
      .set_null(comment_actions::like_score)
      .set_null(comment_actions::liked_at)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)
  }

  async fn remove_likes_in_community(
    pool: &mut DbPool<'_>,
    creator_id: PersonId,
    community_id: CommunityId,
  ) -> LemmyResult<UpleteCount> {
    let comment_ids =
      Comment::creator_comment_ids_in_community(pool, creator_id, community_id).await?;

    let conn = &mut get_conn(pool).await?;

    uplete(comment_actions::table.filter(comment_actions::comment_id.eq_any(comment_ids.clone())))
      .set_null(comment_actions::like_score)
      .set_null(comment_actions::liked_at)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)
  }
}

impl Saveable for CommentActions {
  type Form = CommentSavedForm;
  async fn save(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(comment_actions::table)
      .values(form)
      .on_conflict((comment_actions::comment_id, comment_actions::person_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntSaveComment)
  }
  async fn unsave(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<UpleteCount> {
    let conn = &mut get_conn(pool).await?;
    uplete(comment_actions::table.find((form.person_id, form.comment_id)))
      .set_null(comment_actions::saved_at)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntSaveComment)
  }
}

impl CommentActions {
  pub async fn read(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
    person_id: PersonId,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    comment_actions::table
      .find((person_id, comment_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn fill_comment_like_history(pool: &mut DbPool<'_>) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;

    info!("Filling comment_like history into comment_actions...");

    // Get the total count of comment_like rows, to show progress
    let comment_like_count = comment_like::table
      .select(count_star())
      .first::<i64>(conn)
      .await?;

    let mut processed_rows = 0;

    while processed_rows < comment_like_count {
      let rows_inserted = conn
        .run_transaction(|conn| {
          async move {
            // Select and map into comment like forms
            let forms = comment_like::table
              .order_by(comment_like::published.desc())
              .limit(DB_BATCH_SIZE)
              .get_results::<(PersonId, CommentId, i16, DateTime<Utc>)>(conn)
              .await?
              .iter()
              .map(|cl| CommentLikeForm {
                person_id: cl.0,
                comment_id: cl.1,
                like_score: cl.2,
                liked_at: cl.3,
              })
              .collect::<Vec<CommentLikeForm>>();

            // When this is None, the scanning is complete
            let last_scanned_timestamp = forms.last().map(|f| f.liked_at);

            let inserted_count = insert_into(comment_actions::table)
              .values(forms)
              .on_conflict((comment_actions::comment_id, comment_actions::person_id))
              .do_update()
              .set((
                comment_actions::like_score.eq(excluded(comment_actions::like_score)),
                comment_actions::liked_at.eq(excluded(comment_actions::liked_at)),
              ))
              .execute(conn)
              .await?;

            if let Some(last_scanned_timestamp) = last_scanned_timestamp {
              // Update the history status
              let history_form = HistoryStatusUpdateForm {
                last_scanned_timestamp: Some(Some(last_scanned_timestamp)),
                last_scanned_id: None,
              };
              HistoryStatus::update_conn(
                conn,
                ("comment_like".into(), "comment_actions".into()),
                &history_form,
              )
              .await?;

              // Delete those rows from comment_like
              delete(
                comment_like::table.filter(comment_like::published.gt(last_scanned_timestamp)),
              )
              .execute(conn)
              .await?;
            }

            Ok(inserted_count)
          }
          .scope_boxed()
        })
        .await?;

      processed_rows += i64::try_from(rows_inserted)?;
      let pct_complete = processed_rows * 100 / comment_like_count;
      info!(
        "comment_like -> comment_actions: {processed_rows} / {comment_like_count} , {pct_complete}% complete"
      );
    }

    info!("Finished filling comment_like history into comment_actions.");
    Ok(())
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::{
    newtypes::LanguageId,
    source::{
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    traits::{Crud, Likeable, Saveable},
    utils::{build_db_pool_for_tests, RANK_DEFAULT},
  };
  use diesel_ltree::Ltree;
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

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "terry");

    let inserted_person = Person::create(pool, &new_person).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "test community".to_string(),
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

    let expected_comment = Comment {
      id: inserted_comment.id,
      content: "A test comment".into(),
      creator_id: inserted_person.id,
      post_id: inserted_post.id,
      removed: false,
      deleted: false,
      path: Ltree(format!("0.{}", inserted_comment.id)),
      published_at: inserted_comment.published_at,
      updated_at: None,
      ap_id: Url::parse(&format!(
        "https://lemmy-alpha/comment/{}",
        inserted_comment.id
      ))?
      .into(),
      distinguished: false,
      local: true,
      language_id: LanguageId::default(),
      child_count: 1,
      controversy_rank: 0.0,
      downvotes: 0,
      upvotes: 1,
      score: 1,
      hot_rank: RANK_DEFAULT,
      report_count: 0,
      unresolved_report_count: 0,
      federation_pending: false,
    };

    let child_comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A child comment".into(),
    );
    let inserted_child_comment =
      Comment::create(pool, &child_comment_form, Some(&inserted_comment.path)).await?;

    // Comment Like
    let comment_like_form = CommentLikeForm::new(inserted_person.id, inserted_comment.id, 1);

    let inserted_comment_like = CommentActions::like(pool, &comment_like_form).await?;
    assert_eq!(Some(1), inserted_comment_like.like_score);

    // Comment Saved
    let comment_saved_form = CommentSavedForm::new(inserted_person.id, inserted_comment.id);
    let inserted_comment_saved = CommentActions::save(pool, &comment_saved_form).await?;
    assert!(inserted_comment_saved.saved_at.is_some());

    let comment_update_form = CommentUpdateForm {
      content: Some("A test comment".into()),
      ..Default::default()
    };

    let updated_comment = Comment::update(pool, inserted_comment.id, &comment_update_form).await?;

    let read_comment = Comment::read(pool, inserted_comment.id).await?;
    let like_removed =
      CommentActions::remove_like(pool, inserted_person.id, inserted_comment.id).await?;
    let saved_removed = CommentActions::unsave(pool, &comment_saved_form).await?;
    let num_deleted = Comment::delete(pool, inserted_comment.id).await?;
    Comment::delete(pool, inserted_child_comment.id).await?;
    Post::delete(pool, inserted_post.id).await?;
    Community::delete(pool, inserted_community.id).await?;
    Person::delete(pool, inserted_person.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    assert_eq!(expected_comment, read_comment);
    assert_eq!(expected_comment, updated_comment);
    assert_eq!(
      format!("0.{}.{}", expected_comment.id, inserted_child_comment.id),
      inserted_child_comment.path.0,
    );
    assert_eq!(UpleteCount::only_updated(1), like_removed);
    assert_eq!(UpleteCount::only_deleted(1), saved_removed);
    assert_eq!(1, num_deleted);

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_aggregates() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "thommy_comment_agg");

    let inserted_person = Person::create(pool, &new_person).await?;

    let another_person = PersonInsertForm::test_form(inserted_instance.id, "jerry_comment_agg");

    let another_inserted_person = Person::create(pool, &another_person).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "TIL_comment_agg".into(),
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
    let _inserted_child_comment =
      Comment::create(pool, &child_comment_form, Some(&inserted_comment.path)).await?;

    let comment_like = CommentLikeForm::new(inserted_person.id, inserted_comment.id, 1);

    CommentActions::like(pool, &comment_like).await?;

    let comment_aggs_before_delete = Comment::read(pool, inserted_comment.id).await?;

    assert_eq!(1, comment_aggs_before_delete.score);
    assert_eq!(1, comment_aggs_before_delete.upvotes);
    assert_eq!(0, comment_aggs_before_delete.downvotes);

    // Add a post dislike from the other person
    let comment_dislike = CommentLikeForm::new(another_inserted_person.id, inserted_comment.id, -1);

    CommentActions::like(pool, &comment_dislike).await?;

    let comment_aggs_after_dislike = Comment::read(pool, inserted_comment.id).await?;

    assert_eq!(0, comment_aggs_after_dislike.score);
    assert_eq!(1, comment_aggs_after_dislike.upvotes);
    assert_eq!(1, comment_aggs_after_dislike.downvotes);

    // Remove the first comment like
    CommentActions::remove_like(pool, inserted_person.id, inserted_comment.id).await?;
    let after_like_remove = Comment::read(pool, inserted_comment.id).await?;
    assert_eq!(-1, after_like_remove.score);
    assert_eq!(0, after_like_remove.upvotes);
    assert_eq!(1, after_like_remove.downvotes);

    // Remove the parent post
    Post::delete(pool, inserted_post.id).await?;

    // Should be none found, since the post was deleted
    let after_delete = Comment::read(pool, inserted_comment.id).await;
    assert!(after_delete.is_err());

    // This should delete all the associated rows, and fire triggers
    Person::delete(pool, another_inserted_person.id).await?;
    let person_num_deleted = Person::delete(pool, inserted_person.id).await?;
    assert_eq!(1, person_num_deleted);

    // Delete the community
    let community_num_deleted = Community::delete(pool, inserted_community.id).await?;
    assert_eq!(1, community_num_deleted);

    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }
}
