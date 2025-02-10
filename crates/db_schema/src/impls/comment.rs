use crate::{
  diesel::{DecoratableTarget, OptionalExtension},
  impls::local_user::local_user_can_mod,
  newtypes::{CommentId, DbUrl, PersonId},
  schema::{comment, comment_actions},
  source::comment::{
    Comment,
    CommentInsertForm,
    CommentLike,
    CommentLikeForm,
    CommentSaved,
    CommentSavedForm,
    CommentUpdateForm,
  },
  traits::{Crud, Likeable, Saveable},
  utils::{functions::coalesce, get_conn, now, uplete, DbPool, DELETED_REPLACEMENT_TEXT},
};
use chrono::{DateTime, Utc};
use diesel::{
  dsl::{case_when, insert_into, not},
  expression::SelectableHelper,
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use diesel_ltree::Ltree;
use lemmy_utils::{error::LemmyResult, settings::structs::Settings};
use url::Url;

impl Comment {
  pub async fn permadelete_for_creator(
    pool: &mut DbPool<'_>,
    for_creator_id: PersonId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;

    diesel::update(comment::table.filter(comment::creator_id.eq(for_creator_id)))
      .set((
        comment::content.eq(DELETED_REPLACEMENT_TEXT),
        comment::deleted.eq(true),
        comment::updated.eq(Utc::now()),
      ))
      .get_results::<Self>(conn)
      .await
  }

  pub async fn update_removed_for_creator(
    pool: &mut DbPool<'_>,
    for_creator_id: PersonId,
    removed: bool,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(comment::table.filter(comment::creator_id.eq(for_creator_id)))
      .set((
        comment::removed.eq(removed),
        comment::updated.eq(Utc::now()),
      ))
      .get_results::<Self>(conn)
      .await
  }

  pub async fn create(
    pool: &mut DbPool<'_>,
    comment_form: &CommentInsertForm,
    parent_path: Option<&Ltree>,
  ) -> Result<Comment, Error> {
    Self::insert_apub(pool, None, comment_form, parent_path).await
  }

  pub async fn insert_apub(
    pool: &mut DbPool<'_>,
    timestamp: Option<DateTime<Utc>>,
    comment_form: &CommentInsertForm,
    parent_path: Option<&Ltree>,
  ) -> Result<Comment, Error> {
    let conn = &mut get_conn(pool).await?;
    let comment_form = (comment_form, parent_path.map(|p| comment::path.eq(p)));

    if let Some(timestamp) = timestamp {
      insert_into(comment::table)
        .values(comment_form)
        .on_conflict(comment::ap_id)
        .filter_target(coalesce(comment::updated, comment::published).lt(timestamp))
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
  }

  pub async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: Url,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let object_id: DbUrl = object_id.into();
    comment::table
      .filter(comment::ap_id.eq(object_id))
      .first(conn)
      .await
      .optional()
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

  pub fn local_url(&self, settings: &Settings) -> LemmyResult<DbUrl> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/comment/{}", self.id))?.into())
  }
}

/// Selects the comment columns, but gives an empty string for content when
/// deleted or removed, and you're not a mod/admin.
#[diesel::dsl::auto_type]
pub fn comment_select_remove_deletes() -> _ {
  let deleted_or_removed = comment::deleted.or(comment::removed);

  // You can only view the content if it hasn't been removed, or you can mod.
  let can_view_content = not(deleted_or_removed).or(local_user_can_mod());
  let content = case_when(can_view_content, comment::content).otherwise("");

  (
    comment::id,
    comment::creator_id,
    comment::post_id,
    content,
    comment::removed,
    comment::published,
    comment::updated,
    comment::deleted,
    comment::ap_id,
    comment::local,
    comment::path,
    comment::distinguished,
    comment::language_id,
  )
}

#[async_trait]
impl Crud for Comment {
  type InsertForm = CommentInsertForm;
  type UpdateForm = CommentUpdateForm;
  type IdType = CommentId;

  /// Use [[Comment::create]]
  async fn create(pool: &mut DbPool<'_>, comment_form: &Self::InsertForm) -> Result<Self, Error> {
    debug_assert!(false);
    Comment::create(pool, comment_form, None).await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
    comment_form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(comment::table.find(comment_id))
      .set(comment_form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Likeable for CommentLike {
  type Form = CommentLikeForm;
  type IdType = CommentId;
  async fn like(pool: &mut DbPool<'_>, comment_like_form: &CommentLikeForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let comment_like_form = (
      comment_like_form,
      comment_actions::liked.eq(now().nullable()),
    );
    insert_into(comment_actions::table)
      .values(comment_like_form)
      .on_conflict((comment_actions::comment_id, comment_actions::person_id))
      .do_update()
      .set(comment_like_form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
  }
  async fn remove(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    comment_id: CommentId,
  ) -> Result<uplete::Count, Error> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(comment_actions::table.find((person_id, comment_id)))
      .set_null(comment_actions::like_score)
      .set_null(comment_actions::liked)
      .get_result(conn)
      .await
  }
}

#[async_trait]
impl Saveable for CommentSaved {
  type Form = CommentSavedForm;
  async fn save(
    pool: &mut DbPool<'_>,
    comment_saved_form: &CommentSavedForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(comment_actions::table)
      .values(comment_saved_form)
      .on_conflict((comment_actions::comment_id, comment_actions::person_id))
      .do_update()
      .set(comment_saved_form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
  }
  async fn unsave(
    pool: &mut DbPool<'_>,
    comment_saved_form: &CommentSavedForm,
  ) -> Result<uplete::Count, Error> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(
      comment_actions::table.find((comment_saved_form.person_id, comment_saved_form.comment_id)),
    )
    .set_null(comment_actions::saved)
    .get_result(conn)
    .await
  }
}

#[cfg(test)]
mod tests {

  use crate::{
    newtypes::LanguageId,
    source::{
      comment::{
        Comment,
        CommentInsertForm,
        CommentLike,
        CommentLikeForm,
        CommentSaved,
        CommentSavedForm,
        CommentUpdateForm,
      },
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    traits::{Crud, Likeable, Saveable},
    utils::{build_db_pool_for_tests, uplete},
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
      published: inserted_comment.published,
      updated: None,
      ap_id: Url::parse(&format!(
        "https://lemmy-alpha/comment/{}",
        inserted_comment.id
      ))?
      .into(),
      distinguished: false,
      local: true,
      language_id: LanguageId::default(),
    };

    let child_comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A child comment".into(),
    );
    let inserted_child_comment =
      Comment::create(pool, &child_comment_form, Some(&inserted_comment.path)).await?;

    // Comment Like
    let comment_like_form = CommentLikeForm {
      comment_id: inserted_comment.id,
      person_id: inserted_person.id,
      score: 1,
    };

    let inserted_comment_like = CommentLike::like(pool, &comment_like_form).await?;

    let expected_comment_like = CommentLike {
      comment_id: inserted_comment.id,
      person_id: inserted_person.id,
      published: inserted_comment_like.published,
      score: 1,
    };

    // Comment Saved
    let comment_saved_form = CommentSavedForm::new(inserted_comment.id, inserted_person.id);
    let inserted_comment_saved = CommentSaved::save(pool, &comment_saved_form).await?;

    let expected_comment_saved = CommentSaved {
      comment_id: inserted_comment.id,
      person_id: inserted_person.id,
      published: inserted_comment_saved.published,
    };

    let comment_update_form = CommentUpdateForm {
      content: Some("A test comment".into()),
      ..Default::default()
    };

    let updated_comment = Comment::update(pool, inserted_comment.id, &comment_update_form).await?;

    let read_comment = Comment::read(pool, inserted_comment.id).await?;
    let like_removed = CommentLike::remove(pool, inserted_person.id, inserted_comment.id).await?;
    let saved_removed = CommentSaved::unsave(pool, &comment_saved_form).await?;
    let num_deleted = Comment::delete(pool, inserted_comment.id).await?;
    Comment::delete(pool, inserted_child_comment.id).await?;
    Post::delete(pool, inserted_post.id).await?;
    Community::delete(pool, inserted_community.id).await?;
    Person::delete(pool, inserted_person.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    assert_eq!(expected_comment, read_comment);
    assert_eq!(expected_comment, inserted_comment);
    assert_eq!(expected_comment, updated_comment);
    assert_eq!(expected_comment_like, inserted_comment_like);
    assert_eq!(expected_comment_saved, inserted_comment_saved);
    assert_eq!(
      format!("0.{}.{}", expected_comment.id, inserted_child_comment.id),
      inserted_child_comment.path.0,
    );
    assert_eq!(uplete::Count::only_updated(1), like_removed);
    assert_eq!(uplete::Count::only_deleted(1), saved_removed);
    assert_eq!(1, num_deleted);

    Ok(())
  }
}
