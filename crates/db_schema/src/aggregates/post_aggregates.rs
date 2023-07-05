use crate::{
  aggregates::structs::PostAggregates,
  newtypes::PostId,
  schema::post_aggregates,
  utils::{functions::hot_rank, GetConn},
};
use diesel::{result::Error, ExpressionMethods, QueryDsl};
use lemmy_db_schema::utils::RunQueryDsl;

impl PostAggregates {
  pub async fn read(mut conn: impl GetConn, post_id: PostId) -> Result<Self, Error> {
    post_aggregates::table
      .filter(post_aggregates::post_id.eq(post_id))
      .first::<Self>(conn)
      .await
  }

  pub async fn update_hot_rank(mut conn: impl GetConn, post_id: PostId) -> Result<Self, Error> {
    diesel::update(post_aggregates::table)
      .filter(post_aggregates::post_id.eq(post_id))
      .set((
        post_aggregates::hot_rank.eq(hot_rank(post_aggregates::score, post_aggregates::published)),
        post_aggregates::hot_rank_active.eq(hot_rank(
          post_aggregates::score,
          post_aggregates::newest_comment_time_necro,
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
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm, PostLike, PostLikeForm},
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
      .name("thommy_community_agg".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(conn, &new_person).await.unwrap();

    let another_person = PersonInsertForm::builder()
      .name("jerry_community_agg".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let another_inserted_person = Person::create(conn, &another_person).await.unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("TIL_community_agg".into())
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

    let inserted_child_comment = Comment::create(
      conn,
      &child_comment_form,
      Some(&inserted_comment.path),
    )
    .await
    .unwrap();

    let post_like = PostLikeForm {
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      score: 1,
    };

    PostLike::like(conn, &post_like).await.unwrap();

    let post_aggs_before_delete = PostAggregates::read(conn, inserted_post.id)
      .await
      .unwrap();

    assert_eq!(2, post_aggs_before_delete.comments);
    assert_eq!(1, post_aggs_before_delete.score);
    assert_eq!(1, post_aggs_before_delete.upvotes);
    assert_eq!(0, post_aggs_before_delete.downvotes);

    // Add a post dislike from the other person
    let post_dislike = PostLikeForm {
      post_id: inserted_post.id,
      person_id: another_inserted_person.id,
      score: -1,
    };

    PostLike::like(conn, &post_dislike).await.unwrap();

    let post_aggs_after_dislike = PostAggregates::read(conn, inserted_post.id)
      .await
      .unwrap();

    assert_eq!(2, post_aggs_after_dislike.comments);
    assert_eq!(0, post_aggs_after_dislike.score);
    assert_eq!(1, post_aggs_after_dislike.upvotes);
    assert_eq!(1, post_aggs_after_dislike.downvotes);

    // Remove the comments
    Comment::delete(conn, inserted_comment.id)
      .await
      .unwrap();
    Comment::delete(conn, inserted_child_comment.id)
      .await
      .unwrap();
    let after_comment_delete = PostAggregates::read(conn, inserted_post.id)
      .await
      .unwrap();
    assert_eq!(0, after_comment_delete.comments);
    assert_eq!(0, after_comment_delete.score);
    assert_eq!(1, after_comment_delete.upvotes);
    assert_eq!(1, after_comment_delete.downvotes);

    // Remove the first post like
    PostLike::remove(conn, inserted_person.id, inserted_post.id)
      .await
      .unwrap();
    let after_like_remove = PostAggregates::read(conn, inserted_post.id)
      .await
      .unwrap();
    assert_eq!(0, after_like_remove.comments);
    assert_eq!(-1, after_like_remove.score);
    assert_eq!(0, after_like_remove.upvotes);
    assert_eq!(1, after_like_remove.downvotes);

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

    // Should be none found, since the creator was deleted
    let after_delete = PostAggregates::read(conn, inserted_post.id).await;
    assert!(after_delete.is_err());

    Instance::delete(conn, inserted_instance.id)
      .await
      .unwrap();
  }
}
