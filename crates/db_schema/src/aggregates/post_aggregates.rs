use crate::{
  aggregates::structs::PostAggregates,
  newtypes::PostId,
  schema::{community_aggregates, post, post_aggregates},
  utils::{
    functions::{hot_rank, scaled_rank},
    get_conn,
    DbPool,
  },
};
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;

impl PostAggregates {
  pub async fn read(pool: &mut DbPool<'_>, post_id: PostId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    post_aggregates::table.find(post_id).first(conn).await
  }

  pub async fn update_ranks(pool: &mut DbPool<'_>, post_id: PostId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    // Diesel can't update based on a join, which is necessary for the scaled_rank
    // https://github.com/diesel-rs/diesel/issues/1478
    // Just select the users_active_month manually for now, since its a single post anyway
    let users_active_month = community_aggregates::table
      .select(community_aggregates::users_active_month)
      .inner_join(post::table.on(community_aggregates::community_id.eq(post::community_id)))
      .filter(post::id.eq(post_id))
      .first::<i64>(conn)
      .await?;

    diesel::update(post_aggregates::table.find(post_id))
      .set((
        post_aggregates::hot_rank.eq(hot_rank(post_aggregates::score, post_aggregates::published)),
        post_aggregates::hot_rank_active.eq(hot_rank(
          post_aggregates::score,
          post_aggregates::newest_comment_time_necro,
        )),
        post_aggregates::scaled_rank.eq(scaled_rank(
          post_aggregates::score,
          post_aggregates::published,
          users_active_month,
        )),
      ))
      .get_result::<Self>(conn)
      .await
  }
}

#[cfg(test)]
mod tests {

  use crate::{
    aggregates::post_aggregates::PostAggregates,
    source::{
      comment::{Comment, CommentInsertForm, CommentUpdateForm},
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

    PostLike::like(pool, &post_like).await?;

    let post_aggs_before_delete = PostAggregates::read(pool, inserted_post.id).await?;

    assert_eq!(2, post_aggs_before_delete.comments);
    assert_eq!(1, post_aggs_before_delete.score);
    assert_eq!(1, post_aggs_before_delete.upvotes);
    assert_eq!(0, post_aggs_before_delete.downvotes);

    // Add a post dislike from the other person
    let post_dislike = PostLikeForm::new(inserted_post.id, another_inserted_person.id, -1);

    PostLike::like(pool, &post_dislike).await?;

    let post_aggs_after_dislike = PostAggregates::read(pool, inserted_post.id).await?;

    assert_eq!(2, post_aggs_after_dislike.comments);
    assert_eq!(0, post_aggs_after_dislike.score);
    assert_eq!(1, post_aggs_after_dislike.upvotes);
    assert_eq!(1, post_aggs_after_dislike.downvotes);

    // Remove the comments
    Comment::delete(pool, inserted_comment.id).await?;
    Comment::delete(pool, inserted_child_comment.id).await?;
    let after_comment_delete = PostAggregates::read(pool, inserted_post.id).await?;
    assert_eq!(0, after_comment_delete.comments);
    assert_eq!(0, after_comment_delete.score);
    assert_eq!(1, after_comment_delete.upvotes);
    assert_eq!(1, after_comment_delete.downvotes);

    // Remove the first post like
    PostLike::remove(pool, inserted_person.id, inserted_post.id).await?;
    let after_like_remove = PostAggregates::read(pool, inserted_post.id).await?;
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
    let after_delete = PostAggregates::read(pool, inserted_post.id).await;
    assert!(after_delete.is_err());

    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_soft_delete() -> Result<(), Error> {
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

    let post_aggregates_before = PostAggregates::read(pool, inserted_post.id).await?;
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

    let post_aggregates_after_remove = PostAggregates::read(pool, inserted_post.id).await?;
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

    let post_aggregates_after_delete = PostAggregates::read(pool, inserted_post.id).await?;
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

    let post_aggregates_after_delete_remove = PostAggregates::read(pool, inserted_post.id).await?;
    assert_eq!(0, post_aggregates_after_delete_remove.comments);

    Comment::delete(pool, inserted_comment.id).await?;
    Post::delete(pool, inserted_post.id).await?;
    Person::delete(pool, inserted_person.id).await?;
    Community::delete(pool, inserted_community.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }
}
