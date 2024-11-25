use crate::{
  aggregates::structs::PersonAggregates,
  newtypes::PersonId,
  schema::person_aggregates,
  utils::{get_conn, DbPool},
};
use diesel::{result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

impl PersonAggregates {
  pub async fn read(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    person_aggregates::table.find(person_id).first(conn).await
  }
}

#[cfg(test)]
mod tests {

  use crate::{
    aggregates::person_aggregates::PersonAggregates,
    source::{
      comment::{Comment, CommentInsertForm, CommentLike, CommentLikeForm, CommentUpdateForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm, PostLike, PostLikeForm},
    },
    traits::{Crud, Likeable},
    utils::build_db_pool_for_tests,
  };
  use diesel::result::Error;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() -> Result<(), Error> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "thommy_user_agg");

    let inserted_person = Person::create(pool, &new_person).await?;

    let another_person = PersonInsertForm::test_form(inserted_instance.id, "jerry_user_agg");

    let another_inserted_person = Person::create(pool, &another_person).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "TIL_site_agg".into(),
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

    let post_like = PostLikeForm::new(inserted_post.id, inserted_person.id, 1);
    let _inserted_post_like = PostLike::like(pool, &post_like).await?;

    let comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );
    let inserted_comment = Comment::create(pool, &comment_form, None).await?;

    let mut comment_like = CommentLikeForm {
      comment_id: inserted_comment.id,
      person_id: inserted_person.id,
      score: 1,
    };

    let _inserted_comment_like = CommentLike::like(pool, &comment_like).await?;

    let child_comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );
    let inserted_child_comment =
      Comment::create(pool, &child_comment_form, Some(&inserted_comment.path)).await?;

    let child_comment_like = CommentLikeForm {
      comment_id: inserted_child_comment.id,
      person_id: another_inserted_person.id,
      score: 1,
    };

    let _inserted_child_comment_like = CommentLike::like(pool, &child_comment_like).await?;

    let person_aggregates_before_delete = PersonAggregates::read(pool, inserted_person.id).await?;

    assert_eq!(1, person_aggregates_before_delete.post_count);
    assert_eq!(1, person_aggregates_before_delete.post_score);
    assert_eq!(2, person_aggregates_before_delete.comment_count);
    assert_eq!(2, person_aggregates_before_delete.comment_score);

    // Remove a post like
    PostLike::remove(pool, inserted_person.id, inserted_post.id).await?;
    let after_post_like_remove = PersonAggregates::read(pool, inserted_person.id).await?;
    assert_eq!(0, after_post_like_remove.post_score);

    Comment::update(
      pool,
      inserted_comment.id,
      &CommentUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;
    Comment::update(
      pool,
      inserted_child_comment.id,
      &CommentUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let after_parent_comment_removed = PersonAggregates::read(pool, inserted_person.id).await?;
    assert_eq!(0, after_parent_comment_removed.comment_count);
    // TODO: fix person aggregate comment score calculation
    // assert_eq!(0, after_parent_comment_removed.comment_score);

    // Remove a parent comment (the scores should also be removed)
    Comment::delete(pool, inserted_comment.id).await?;
    Comment::delete(pool, inserted_child_comment.id).await?;
    let after_parent_comment_delete = PersonAggregates::read(pool, inserted_person.id).await?;
    assert_eq!(0, after_parent_comment_delete.comment_count);
    // TODO: fix person aggregate comment score calculation
    // assert_eq!(0, after_parent_comment_delete.comment_score);

    // Add in the two comments again, then delete the post.
    let new_parent_comment = Comment::create(pool, &comment_form, None).await?;
    let _new_child_comment =
      Comment::create(pool, &child_comment_form, Some(&new_parent_comment.path)).await?;
    comment_like.comment_id = new_parent_comment.id;
    CommentLike::like(pool, &comment_like).await?;
    let after_comment_add = PersonAggregates::read(pool, inserted_person.id).await?;
    assert_eq!(2, after_comment_add.comment_count);
    // TODO: fix person aggregate comment score calculation
    // assert_eq!(1, after_comment_add.comment_score);

    Post::delete(pool, inserted_post.id).await?;
    let after_post_delete = PersonAggregates::read(pool, inserted_person.id).await?;
    // TODO: fix person aggregate comment score calculation
    // assert_eq!(0, after_post_delete.comment_score);
    assert_eq!(0, after_post_delete.comment_count);
    assert_eq!(0, after_post_delete.post_score);
    assert_eq!(0, after_post_delete.post_count);

    // This should delete all the associated rows, and fire triggers
    let person_num_deleted = Person::delete(pool, inserted_person.id).await?;
    assert_eq!(1, person_num_deleted);
    Person::delete(pool, another_inserted_person.id).await?;

    // Delete the community
    let community_num_deleted = Community::delete(pool, inserted_community.id).await?;
    assert_eq!(1, community_num_deleted);

    // Should be none found
    let after_delete = PersonAggregates::read(pool, inserted_person.id).await;
    assert!(after_delete.is_err());

    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }
}
