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
    comment_aggregates::table
      .find(comment_id)
      .first::<Self>(conn)
      .await
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
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
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
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person = PersonInsertForm::builder()
      .name("thommy_comment_agg".into())
      .public_key("pubkey".into())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(pool, &new_person).await.unwrap();

    let another_person = PersonInsertForm::builder()
      .name("jerry_comment_agg".into())
      .public_key("pubkey".into())
      .instance_id(inserted_instance.id)
      .build();

    let another_inserted_person = Person::create(pool, &another_person).await.unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("TIL_comment_agg".into())
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

    let comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    let inserted_comment = Comment::create(pool, &comment_form, None).await.unwrap();

    let child_comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    let _inserted_child_comment =
      Comment::create(pool, &child_comment_form, Some(&inserted_comment.path))
        .await
        .unwrap();

    let comment_like = CommentLikeForm {
      comment_id: inserted_comment.id,
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      score: 1,
    };

    CommentLike::like(pool, &comment_like).await.unwrap();

    let comment_aggs_before_delete = CommentAggregates::read(pool, inserted_comment.id)
      .await
      .unwrap();

    assert_eq!(1, comment_aggs_before_delete.score);
    assert_eq!(1, comment_aggs_before_delete.upvotes);
    assert_eq!(0, comment_aggs_before_delete.downvotes);

    // Add a post dislike from the other person
    let comment_dislike = CommentLikeForm {
      comment_id: inserted_comment.id,
      post_id: inserted_post.id,
      person_id: another_inserted_person.id,
      score: -1,
    };

    CommentLike::like(pool, &comment_dislike).await.unwrap();

    let comment_aggs_after_dislike = CommentAggregates::read(pool, inserted_comment.id)
      .await
      .unwrap();

    assert_eq!(0, comment_aggs_after_dislike.score);
    assert_eq!(1, comment_aggs_after_dislike.upvotes);
    assert_eq!(1, comment_aggs_after_dislike.downvotes);

    // Remove the first comment like
    CommentLike::remove(pool, inserted_person.id, inserted_comment.id)
      .await
      .unwrap();
    let after_like_remove = CommentAggregates::read(pool, inserted_comment.id)
      .await
      .unwrap();
    assert_eq!(-1, after_like_remove.score);
    assert_eq!(0, after_like_remove.upvotes);
    assert_eq!(1, after_like_remove.downvotes);

    // Remove the parent post
    Post::delete(pool, inserted_post.id).await.unwrap();

    // Should be none found, since the post was deleted
    let after_delete = CommentAggregates::read(pool, inserted_comment.id).await;
    assert!(after_delete.is_err());

    // This should delete all the associated rows, and fire triggers
    Person::delete(pool, another_inserted_person.id)
      .await
      .unwrap();
    let person_num_deleted = Person::delete(pool, inserted_person.id).await.unwrap();
    assert_eq!(1, person_num_deleted);

    // Delete the community
    let community_num_deleted = Community::delete(pool, inserted_community.id)
      .await
      .unwrap();
    assert_eq!(1, community_num_deleted);

    Instance::delete(pool, inserted_instance.id).await.unwrap();
  }
}
