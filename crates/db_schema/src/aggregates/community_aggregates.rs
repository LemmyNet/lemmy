use crate::{
  aggregates::structs::CommunityAggregates,
  diesel::OptionalExtension,
  newtypes::CommunityId,
  schema::{community_aggregates, community_aggregates::subscribers},
  utils::{get_conn, DbPool},
};
use diesel::{result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl CommunityAggregates {
  pub async fn read(
    pool: &mut DbPool<'_>,
    for_community_id: CommunityId,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    community_aggregates::table
      .find(for_community_id)
      .first(conn)
      .await
      .optional()
  }

  pub async fn update_federated_followers(
    pool: &mut DbPool<'_>,
    for_community_id: CommunityId,
    new_subscribers: i32,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let new_subscribers: i64 = new_subscribers.into();
    diesel::update(community_aggregates::table.find(for_community_id))
      .set(subscribers.eq(new_subscribers))
      .get_result(conn)
      .await
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use crate::{
    aggregates::community_aggregates::CommunityAggregates,
    source::{
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityFollower, CommunityFollowerForm, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    traits::{Crud, Followable},
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

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "thommy_community_agg");

    let inserted_person = Person::create(pool, &new_person).await.unwrap();

    let another_person = PersonInsertForm::test_form(inserted_instance.id, "jerry_community_agg");

    let another_inserted_person = Person::create(pool, &another_person).await.unwrap();

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "TIL_community_agg".into(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let inserted_community = Community::create(pool, &new_community).await.unwrap();

    let another_community = CommunityInsertForm::new(
      inserted_instance.id,
      "TIL_community_agg_2".into(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let another_inserted_community = Community::create(pool, &another_community).await.unwrap();

    let first_person_follow = CommunityFollowerForm {
      community_id: inserted_community.id,
      person_id: inserted_person.id,
      pending: false,
    };

    CommunityFollower::follow(pool, &first_person_follow)
      .await
      .unwrap();

    let second_person_follow = CommunityFollowerForm {
      community_id: inserted_community.id,
      person_id: another_inserted_person.id,
      pending: false,
    };

    CommunityFollower::follow(pool, &second_person_follow)
      .await
      .unwrap();

    let another_community_follow = CommunityFollowerForm {
      community_id: another_inserted_community.id,
      person_id: inserted_person.id,
      pending: false,
    };

    CommunityFollower::follow(pool, &another_community_follow)
      .await
      .unwrap();

    let new_post = PostInsertForm::new(
      "A test post".into(),
      inserted_person.id,
      inserted_community.id,
    );
    let inserted_post = Post::create(pool, &new_post).await.unwrap();

    let comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );
    let inserted_comment = Comment::create(pool, &comment_form, None).await.unwrap();

    let child_comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );
    let _inserted_child_comment =
      Comment::create(pool, &child_comment_form, Some(&inserted_comment.path))
        .await
        .unwrap();

    let community_aggregates_before_delete = CommunityAggregates::read(pool, inserted_community.id)
      .await
      .unwrap()
      .unwrap();

    assert_eq!(2, community_aggregates_before_delete.subscribers);
    assert_eq!(2, community_aggregates_before_delete.subscribers_local);
    assert_eq!(1, community_aggregates_before_delete.posts);
    assert_eq!(2, community_aggregates_before_delete.comments);

    // Test the other community
    let another_community_aggs = CommunityAggregates::read(pool, another_inserted_community.id)
      .await
      .unwrap()
      .unwrap();
    assert_eq!(1, another_community_aggs.subscribers);
    assert_eq!(1, another_community_aggs.subscribers_local);
    assert_eq!(0, another_community_aggs.posts);
    assert_eq!(0, another_community_aggs.comments);

    // Unfollow test
    CommunityFollower::unfollow(pool, &second_person_follow)
      .await
      .unwrap();
    let after_unfollow = CommunityAggregates::read(pool, inserted_community.id)
      .await
      .unwrap()
      .unwrap();
    assert_eq!(1, after_unfollow.subscribers);
    assert_eq!(1, after_unfollow.subscribers_local);

    // Follow again just for the later tests
    CommunityFollower::follow(pool, &second_person_follow)
      .await
      .unwrap();
    let after_follow_again = CommunityAggregates::read(pool, inserted_community.id)
      .await
      .unwrap()
      .unwrap();
    assert_eq!(2, after_follow_again.subscribers);
    assert_eq!(2, after_follow_again.subscribers_local);

    // Remove a parent post (the comment count should also be 0)
    Post::delete(pool, inserted_post.id).await.unwrap();
    let after_parent_post_delete = CommunityAggregates::read(pool, inserted_community.id)
      .await
      .unwrap()
      .unwrap();
    assert_eq!(0, after_parent_post_delete.comments);
    assert_eq!(0, after_parent_post_delete.posts);

    // Remove the 2nd person
    Person::delete(pool, another_inserted_person.id)
      .await
      .unwrap();
    let after_person_delete = CommunityAggregates::read(pool, inserted_community.id)
      .await
      .unwrap()
      .unwrap();
    assert_eq!(1, after_person_delete.subscribers);
    assert_eq!(1, after_person_delete.subscribers_local);

    // This should delete all the associated rows, and fire triggers
    let person_num_deleted = Person::delete(pool, inserted_person.id).await.unwrap();
    assert_eq!(1, person_num_deleted);

    // Delete the community
    let community_num_deleted = Community::delete(pool, inserted_community.id)
      .await
      .unwrap();
    assert_eq!(1, community_num_deleted);

    let another_community_num_deleted = Community::delete(pool, another_inserted_community.id)
      .await
      .unwrap();
    assert_eq!(1, another_community_num_deleted);

    // Should be none found, since the creator was deleted
    let after_delete = CommunityAggregates::read(pool, inserted_community.id)
      .await
      .unwrap();
    assert!(after_delete.is_none());
  }
}
