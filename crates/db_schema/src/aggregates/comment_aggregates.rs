use crate::{
  aggregates::structs::CommentAggregates,
  newtypes::CommentId,
  schema::comment_aggregates,
  utils::{functions::hot_rank, GetConn},
};
use diesel::{result::Error, ExpressionMethods, QueryDsl};
use lemmy_db_schema::utils::RunQueryDsl;

impl CommentAggregates {
  pub async fn read(mut conn: impl GetConn, comment_id: CommentId) -> Result<Self, Error> {
    comment_aggregates::table
      .filter(comment_aggregates::comment_id.eq(comment_id))
      .first::<Self>(conn)
      .await
  }

  pub async fn update_hot_rank(
    mut conn: impl GetConn,
    comment_id: CommentId,
  ) -> Result<Self, Error> {
    diesel::update(comment_aggregates::table)
      .filter(comment_aggregates::comment_id.eq(comment_id))
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
    utils::build_db_conn_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let mut conn = build_db_conn_for_tests().await;

    let inserted_instance = Instance::read_or_create(conn, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person = PersonInsertForm::builder()
      .name("thommy_comment_agg".into())
      .public_key("pubkey".into())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(conn, &new_person).await.unwrap();

    let another_person = PersonInsertForm::builder()
      .name("jerry_comment_agg".into())
      .public_key("pubkey".into())
      .instance_id(inserted_instance.id)
      .build();

    let another_inserted_person = Person::create(conn, &another_person).await.unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("TIL_comment_agg".into())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(conn, &new_community).await.unwrap();

    let new_post = PostInsertForm::builder()
      .name("A test post".into())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post = Post::create(conn, &new_post).await.unwrap();

    let comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    let inserted_comment = Comment::create(conn, &comment_form, None)
      .await
      .unwrap();

    let child_comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    let _inserted_child_comment = Comment::create(
      conn,
      &child_comment_form,
      Some(&inserted_comment.path),
    )
    .await
    .unwrap();

    let comment_like = CommentLikeForm {
      comment_id: inserted_comment.id,
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      score: 1,
    };

    CommentLike::like(conn, &comment_like).await.unwrap();

    let comment_aggs_before_delete = CommentAggregates::read(conn, inserted_comment.id)
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

    CommentLike::like(conn, &comment_dislike)
      .await
      .unwrap();

    let comment_aggs_after_dislike = CommentAggregates::read(conn, inserted_comment.id)
      .await
      .unwrap();

    assert_eq!(0, comment_aggs_after_dislike.score);
    assert_eq!(1, comment_aggs_after_dislike.upvotes);
    assert_eq!(1, comment_aggs_after_dislike.downvotes);

    // Remove the first comment like
    CommentLike::remove(conn, inserted_person.id, inserted_comment.id)
      .await
      .unwrap();
    let after_like_remove = CommentAggregates::read(conn, inserted_comment.id)
      .await
      .unwrap();
    assert_eq!(-1, after_like_remove.score);
    assert_eq!(0, after_like_remove.upvotes);
    assert_eq!(1, after_like_remove.downvotes);

    // Remove the parent post
    Post::delete(conn, inserted_post.id).await.unwrap();

    // Should be none found, since the post was deleted
    let after_delete = CommentAggregates::read(conn, inserted_comment.id).await;
    assert!(after_delete.is_err());

    // This should delete all the associated rows, and fire triggers
    Person::delete(conn, another_inserted_person.id)
      .await
      .unwrap();
    let person_num_deleted = Person::delete(conn, inserted_person.id)
      .await
      .unwrap();
    assert_eq!(1, person_num_deleted);

    // Delete the community
    let community_num_deleted = Community::delete(conn, inserted_community.id)
      .await
      .unwrap();
    assert_eq!(1, community_num_deleted);

    Instance::delete(conn, inserted_instance.id)
      .await
      .unwrap();
  }
}
