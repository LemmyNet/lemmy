use crate::{aggregates::structs::PersonAggregates, newtypes::PersonId, schema::person_aggregates};
use diesel::{result::Error, *};

impl PersonAggregates {
  pub fn read(conn: &mut PgConnection, person_id: PersonId) -> Result<Self, Error> {
    person_aggregates::table
      .filter(person_aggregates::person_id.eq(person_id))
      .first::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    aggregates::person_aggregates::PersonAggregates,
    source::{
      comment::{Comment, CommentForm, CommentLike, CommentLikeForm},
      community::{Community, CommunityForm},
      person::{Person, PersonForm},
      post::{Post, PostForm, PostLike, PostLikeForm},
    },
    traits::{Crud, Likeable},
    utils::establish_unpooled_connection,
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = &mut establish_unpooled_connection();

    let new_person = PersonForm {
      name: "thommy_user_agg".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let inserted_person = Person::create(conn, &new_person).unwrap();

    let another_person = PersonForm {
      name: "jerry_user_agg".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let another_inserted_person = Person::create(conn, &another_person).unwrap();

    let new_community = CommunityForm {
      name: "TIL_site_agg".into(),
      title: "nada".to_owned(),
      public_key: Some("pubkey".to_string()),
      ..CommunityForm::default()
    };

    let inserted_community = Community::create(conn, &new_community).unwrap();

    let new_post = PostForm {
      name: "A test post".into(),
      creator_id: inserted_person.id,
      community_id: inserted_community.id,
      ..PostForm::default()
    };

    let inserted_post = Post::create(conn, &new_post).unwrap();

    let post_like = PostLikeForm {
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      score: 1,
    };

    let _inserted_post_like = PostLike::like(conn, &post_like).unwrap();

    let comment_form = CommentForm {
      content: "A test comment".into(),
      creator_id: inserted_person.id,
      post_id: inserted_post.id,
      ..CommentForm::default()
    };

    let inserted_comment = Comment::create(conn, &comment_form, None).unwrap();

    let mut comment_like = CommentLikeForm {
      comment_id: inserted_comment.id,
      person_id: inserted_person.id,
      post_id: inserted_post.id,
      score: 1,
    };

    let _inserted_comment_like = CommentLike::like(conn, &comment_like).unwrap();

    let child_comment_form = CommentForm {
      content: "A test comment".into(),
      creator_id: inserted_person.id,
      post_id: inserted_post.id,
      ..CommentForm::default()
    };

    let inserted_child_comment =
      Comment::create(conn, &child_comment_form, Some(&inserted_comment.path)).unwrap();

    let child_comment_like = CommentLikeForm {
      comment_id: inserted_child_comment.id,
      person_id: another_inserted_person.id,
      post_id: inserted_post.id,
      score: 1,
    };

    let _inserted_child_comment_like = CommentLike::like(conn, &child_comment_like).unwrap();

    let person_aggregates_before_delete = PersonAggregates::read(conn, inserted_person.id).unwrap();

    assert_eq!(1, person_aggregates_before_delete.post_count);
    assert_eq!(1, person_aggregates_before_delete.post_score);
    assert_eq!(2, person_aggregates_before_delete.comment_count);
    assert_eq!(2, person_aggregates_before_delete.comment_score);

    // Remove a post like
    PostLike::remove(conn, inserted_person.id, inserted_post.id).unwrap();
    let after_post_like_remove = PersonAggregates::read(conn, inserted_person.id).unwrap();
    assert_eq!(0, after_post_like_remove.post_score);

    // Remove a parent comment (the scores should also be removed)
    Comment::delete(conn, inserted_comment.id).unwrap();
    Comment::delete(conn, inserted_child_comment.id).unwrap();
    let after_parent_comment_delete = PersonAggregates::read(conn, inserted_person.id).unwrap();
    assert_eq!(0, after_parent_comment_delete.comment_count);
    assert_eq!(0, after_parent_comment_delete.comment_score);

    // Add in the two comments again, then delete the post.
    let new_parent_comment = Comment::create(conn, &comment_form, None).unwrap();
    let _new_child_comment =
      Comment::create(conn, &child_comment_form, Some(&new_parent_comment.path)).unwrap();
    comment_like.comment_id = new_parent_comment.id;
    CommentLike::like(conn, &comment_like).unwrap();
    let after_comment_add = PersonAggregates::read(conn, inserted_person.id).unwrap();
    assert_eq!(2, after_comment_add.comment_count);
    assert_eq!(1, after_comment_add.comment_score);

    Post::delete(conn, inserted_post.id).unwrap();
    let after_post_delete = PersonAggregates::read(conn, inserted_person.id).unwrap();
    assert_eq!(0, after_post_delete.comment_score);
    assert_eq!(0, after_post_delete.comment_count);
    assert_eq!(0, after_post_delete.post_score);
    assert_eq!(0, after_post_delete.post_count);

    // This should delete all the associated rows, and fire triggers
    let person_num_deleted = Person::delete(conn, inserted_person.id).unwrap();
    assert_eq!(1, person_num_deleted);
    Person::delete(conn, another_inserted_person.id).unwrap();

    // Delete the community
    let community_num_deleted = Community::delete(conn, inserted_community.id).unwrap();
    assert_eq!(1, community_num_deleted);

    // Should be none found
    let after_delete = PersonAggregates::read(conn, inserted_person.id);
    assert!(after_delete.is_err());
  }
}
