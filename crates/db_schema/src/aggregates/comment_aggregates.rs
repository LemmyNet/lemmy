use crate::{
  aggregates::structs::CommentAggregates,
  newtypes::CommentId,
  schema::comment_aggregates,
  utils::{functions::hot_rank, get_conn, DbPool},
};
use diesel::{result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl CommentAggregates {
  pub async fn read(pool: &mut DbPool<'_>, comment_id: CommentId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    comment_aggregates::table.find(comment_id).first(conn).await
  }

  pub async fn update_hot_rank(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    diesel::update(comment_aggregates::table.find(comment_id))
      .set(comment_aggregates::hot_rank.eq(hot_rank(
        comment_aggregates::score,
        comment_aggregates::published,
      )))
      .get_result::<Self>(conn)
      .await
  }
}

#[cfg(test)]
mod tests {

  use crate::{
    aggregates::comment_aggregates::CommentAggregates,
    source::{
      comment::{Comment, CommentInsertForm, CommentLike, CommentLikeForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
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

    let comment_like = CommentLikeForm {
      comment_id: inserted_comment.id,
      person_id: inserted_person.id,
      score: 1,
    };

    CommentLike::like(pool, &comment_like).await?;

    let comment_aggs_before_delete = CommentAggregates::read(pool, inserted_comment.id).await?;

    assert_eq!(1, comment_aggs_before_delete.score);
    assert_eq!(1, comment_aggs_before_delete.upvotes);
    assert_eq!(0, comment_aggs_before_delete.downvotes);

    // Add a post dislike from the other person
    let comment_dislike = CommentLikeForm {
      comment_id: inserted_comment.id,
      person_id: another_inserted_person.id,
      score: -1,
    };

    CommentLike::like(pool, &comment_dislike).await?;

    let comment_aggs_after_dislike = CommentAggregates::read(pool, inserted_comment.id).await?;

    assert_eq!(0, comment_aggs_after_dislike.score);
    assert_eq!(1, comment_aggs_after_dislike.upvotes);
    assert_eq!(1, comment_aggs_after_dislike.downvotes);

    // Remove the first comment like
    CommentLike::remove(pool, inserted_person.id, inserted_comment.id).await?;
    let after_like_remove = CommentAggregates::read(pool, inserted_comment.id).await?;
    assert_eq!(-1, after_like_remove.score);
    assert_eq!(0, after_like_remove.upvotes);
    assert_eq!(1, after_like_remove.downvotes);

    // Remove the parent post
    Post::delete(pool, inserted_post.id).await?;

    // Should be none found, since the post was deleted
    let after_delete = CommentAggregates::read(pool, inserted_comment.id).await;
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
