use crate::{
  diesel::{DecoratableTarget, OptionalExtension},
  newtypes::{CommentId, CommunityId, DbUrl, InstanceId, PersonId, PostId},
  source::comment::{
    Comment,
    CommentActions,
    CommentInsertForm,
    CommentLikeForm,
    CommentSavedForm,
    CommentUpdateForm,
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
  dsl::{insert_into, not},
  expression::SelectableHelper,
  update,
  ExpressionMethods,
  JoinOnDsl,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use diesel_ltree::{dsl::LtreeExtensions, Ltree};
use diesel_uplete::{uplete, UpleteCount};
use lemmy_db_schema_file::schema::{comment, comment_actions, community, post};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorExt2, LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
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
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
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
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
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
    } else {
      insert_into(comment::table)
        .values(comment_form)
        .get_result::<Self>(conn)
        .await
    }
    .with_lemmy_type(LemmyErrorType::CouldntCreate)
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
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
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

  /// Updates the locked field for a comment and all its children.
  pub async fn update_locked_for_comment_and_children(
    pool: &mut DbPool<'_>,
    comment_path: &Ltree,
    locked: bool,
  ) -> LemmyResult<Vec<Self>> {
    let form = CommentUpdateForm {
      locked: Some(locked),
      ..Default::default()
    };
    Self::update_comment_and_children(pool, comment_path, &form).await
  }

  /// A helper function to update comment and all its children.
  ///
  /// Don't expose so as to make sure you aren't overwriting data.
  async fn update_comment_and_children(
    pool: &mut DbPool<'_>,
    comment_path: &Ltree,
    form: &CommentUpdateForm,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(comment::table)
      .filter(comment::path.contained_by(comment_path))
      .set(form)
      .get_results(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  pub async fn read_ap_ids_for_post(
    post_id: PostId,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Vec<DbUrl>> {
    let conn = &mut get_conn(pool).await?;
    comment::table
      .filter(comment::post_id.eq(post_id))
      .filter(not(comment::deleted))
      .filter(not(comment::removed))
      .filter(not(comment::federation_pending))
      .order_by(comment::id)
      .select(comment::ap_id)
      .get_results(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
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
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl Likeable for CommentActions {
  type Form = CommentLikeForm;
  type IdType = CommentId;

  async fn like(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    validate_like(form.like_score).with_lemmy_type(LemmyErrorType::CouldntCreate)?;

    insert_into(comment_actions::table)
      .values(form)
      .on_conflict((comment_actions::comment_id, comment_actions::person_id))
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
    comment_id: Self::IdType,
  ) -> LemmyResult<UpleteCount> {
    let conn = &mut get_conn(pool).await?;
    uplete(comment_actions::table.find((person_id, comment_id)))
      .set_null(comment_actions::like_score)
      .set_null(comment_actions::liked_at)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
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
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
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
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
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
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }
  async fn unsave(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<UpleteCount> {
    let conn = &mut get_conn(pool).await?;
    uplete(comment_actions::table.find((form.person_id, form.comment_id)))
      .set_null(comment_actions::saved_at)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
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
      locked: false,
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

  #[tokio::test]
  #[serial]
  async fn test_update_children() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "mydomain.tld".to_string()).await?;
    let new_person = PersonInsertForm::test_form(inserted_instance.id, "john");
    let inserted_person = Person::create(pool, &new_person).await?;
    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "test".into(),
      "test".to_owned(),
      "pubkey".to_string(),
    );
    let inserted_community = Community::create(pool, &new_community).await?;

    let new_post = PostInsertForm::new(
      "Post Title".to_string(),
      inserted_person.id,
      inserted_community.id,
    );
    let inserted_post = Post::create(pool, &new_post).await?;

    let parent_comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "Top level".to_string(),
    );
    let inserted_parent_comment = Comment::create(pool, &parent_comment_form, None).await?;

    let child_comment_form =
      CommentInsertForm::new(inserted_person.id, inserted_post.id, "Child".to_string());
    let inserted_child_comment = Comment::create(
      pool,
      &child_comment_form,
      Some(&inserted_parent_comment.path),
    )
    .await?;

    let grandchild_comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "Grandchild".to_string(),
    );
    let _inserted_grandchild_comment = Comment::create(
      pool,
      &grandchild_comment_form,
      Some(&inserted_child_comment.path),
    )
    .await?;

    let lock_form = CommentUpdateForm {
      locked: Some(true),
      ..Default::default()
    };

    let updated_comments =
      Comment::update_comment_and_children(pool, &inserted_parent_comment.path, &lock_form).await?;

    let locked_comments_num = updated_comments.iter().filter(|c| c.locked).count();

    assert_eq!(3, locked_comments_num);

    Ok(())
  }
}
