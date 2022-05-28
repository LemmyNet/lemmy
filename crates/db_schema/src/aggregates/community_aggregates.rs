use crate::{
  aggregates::structs::CommunityAggregates, newtypes::CommunityId, schema::community_aggregates,
};
use diesel::{result::Error, *};

impl CommunityAggregates {
  pub fn read(conn: &mut PgConnection, community_id: CommunityId) -> Result<Self, Error> {
    community_aggregates::table
      .filter(community_aggregates::community_id.eq(community_id))
      .first::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    aggregates::community_aggregates::CommunityAggregates,
    source::{
      comment::{Comment, CommentForm},
      community::{Community, CommunityFollower, CommunityFollowerForm, CommunityForm},
      person::{Person, PersonForm},
      post::{Post, PostForm},
    },
    traits::{Crud, Followable},
    utils::establish_unpooled_connection,
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_person = PersonForm {
      name: "thommy_community_agg".into(),
      ..PersonForm::default()
    };

    let inserted_person = Person::create(&mut conn, &new_person).unwrap();

    let another_person = PersonForm {
      name: "jerry_community_agg".into(),
      ..PersonForm::default()
    };

    let another_inserted_person = Person::create(&mut conn, &another_person).unwrap();

    let new_community = CommunityForm {
      name: "TIL_community_agg".into(),
      title: "nada".to_owned(),
      ..CommunityForm::default()
    };

    let inserted_community = Community::create(&mut conn, &new_community).unwrap();

    let another_community = CommunityForm {
      name: "TIL_community_agg_2".into(),
      title: "nada".to_owned(),
      ..CommunityForm::default()
    };

    let another_inserted_community = Community::create(&mut conn, &another_community).unwrap();

    let first_person_follow = CommunityFollowerForm {
      community_id: inserted_community.id,
      person_id: inserted_person.id,
      pending: false,
    };

    CommunityFollower::follow(&mut conn, &first_person_follow).unwrap();

    let second_person_follow = CommunityFollowerForm {
      community_id: inserted_community.id,
      person_id: another_inserted_person.id,
      pending: false,
    };

    CommunityFollower::follow(&mut conn, &second_person_follow).unwrap();

    let another_community_follow = CommunityFollowerForm {
      community_id: another_inserted_community.id,
      person_id: inserted_person.id,
      pending: false,
    };

    CommunityFollower::follow(&mut conn, &another_community_follow).unwrap();

    let new_post = PostForm {
      name: "A test post".into(),
      creator_id: inserted_person.id,
      community_id: inserted_community.id,
      ..PostForm::default()
    };

    let inserted_post = Post::create(&mut conn, &new_post).unwrap();

    let comment_form = CommentForm {
      content: "A test comment".into(),
      creator_id: inserted_person.id,
      post_id: inserted_post.id,
      ..CommentForm::default()
    };

    let inserted_comment = Comment::create(&mut conn, &comment_form).unwrap();

    let child_comment_form = CommentForm {
      content: "A test comment".into(),
      creator_id: inserted_person.id,
      post_id: inserted_post.id,
      parent_id: Some(inserted_comment.id),
      ..CommentForm::default()
    };

    let _inserted_child_comment = Comment::create(&mut conn, &child_comment_form).unwrap();

    let community_aggregates_before_delete =
      CommunityAggregates::read(&mut conn, inserted_community.id).unwrap();

    assert_eq!(2, community_aggregates_before_delete.subscribers);
    assert_eq!(1, community_aggregates_before_delete.posts);
    assert_eq!(2, community_aggregates_before_delete.comments);

    // Test the other community
    let another_community_aggs =
      CommunityAggregates::read(&mut conn, another_inserted_community.id).unwrap();
    assert_eq!(1, another_community_aggs.subscribers);
    assert_eq!(0, another_community_aggs.posts);
    assert_eq!(0, another_community_aggs.comments);

    // Unfollow test
    CommunityFollower::unfollow(&mut conn, &second_person_follow).unwrap();
    let after_unfollow = CommunityAggregates::read(&mut conn, inserted_community.id).unwrap();
    assert_eq!(1, after_unfollow.subscribers);

    // Follow again just for the later tests
    CommunityFollower::follow(&mut conn, &second_person_follow).unwrap();
    let after_follow_again = CommunityAggregates::read(&mut conn, inserted_community.id).unwrap();
    assert_eq!(2, after_follow_again.subscribers);

    // Remove a parent comment (the comment count should also be 0)
    Post::delete(&mut conn, inserted_post.id).unwrap();
    let after_parent_post_delete =
      CommunityAggregates::read(&mut conn, inserted_community.id).unwrap();
    assert_eq!(0, after_parent_post_delete.comments);
    assert_eq!(0, after_parent_post_delete.posts);

    // Remove the 2nd person
    Person::delete(&mut conn, another_inserted_person.id).unwrap();
    let after_person_delete = CommunityAggregates::read(&mut conn, inserted_community.id).unwrap();
    assert_eq!(1, after_person_delete.subscribers);

    // This should delete all the associated rows, and fire triggers
    let person_num_deleted = Person::delete(&mut conn, inserted_person.id).unwrap();
    assert_eq!(1, person_num_deleted);

    // Delete the community
    let community_num_deleted = Community::delete(&mut conn, inserted_community.id).unwrap();
    assert_eq!(1, community_num_deleted);

    let another_community_num_deleted =
      Community::delete(&mut conn, another_inserted_community.id).unwrap();
    assert_eq!(1, another_community_num_deleted);

    // Should be none found, since the creator was deleted
    let after_delete = CommunityAggregates::read(&mut conn, inserted_community.id);
    assert!(after_delete.is_err());
  }
}
