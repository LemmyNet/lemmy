use crate::{aggregates::structs::PostAggregates, newtypes::PostId, schema::post_aggregates};
use diesel::{result::Error, *};

impl PostAggregates {
  pub fn read(conn: &mut PgConnection, post_id: PostId) -> Result<Self, Error> {
    post_aggregates::table
      .filter(post_aggregates::post_id.eq(post_id))
      .first::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    aggregates::post_aggregates::PostAggregates,
    source::{
      comment::{Comment, CommentForm},
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

    let post_like = PostLikeForm {
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      score: 1,
    };

    PostLike::like(&mut conn, &post_like).unwrap();

    let post_aggs_before_delete = PostAggregates::read(&mut conn, inserted_post.id).unwrap();

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

    PostLike::like(&mut conn, &post_dislike).unwrap();

    let post_aggs_after_dislike = PostAggregates::read(&mut conn, inserted_post.id).unwrap();

    assert_eq!(2, post_aggs_after_dislike.comments);
    assert_eq!(0, post_aggs_after_dislike.score);
    assert_eq!(1, post_aggs_after_dislike.upvotes);
    assert_eq!(1, post_aggs_after_dislike.downvotes);

    // Remove the parent comment
    Comment::delete(&mut conn, inserted_comment.id).unwrap();
    let after_comment_delete = PostAggregates::read(&mut conn, inserted_post.id).unwrap();
    assert_eq!(0, after_comment_delete.comments);
    assert_eq!(0, after_comment_delete.score);
    assert_eq!(1, after_comment_delete.upvotes);
    assert_eq!(1, after_comment_delete.downvotes);

    // Remove the first post like
    PostLike::remove(&mut conn, inserted_person.id, inserted_post.id).unwrap();
    let after_like_remove = PostAggregates::read(&mut conn, inserted_post.id).unwrap();
    assert_eq!(0, after_like_remove.comments);
    assert_eq!(-1, after_like_remove.score);
    assert_eq!(0, after_like_remove.upvotes);
    assert_eq!(1, after_like_remove.downvotes);

    // This should delete all the associated rows, and fire triggers
    Person::delete(&mut conn, another_inserted_person.id).unwrap();
    let person_num_deleted = Person::delete(&mut conn, inserted_person.id).unwrap();
    assert_eq!(1, person_num_deleted);

    // Delete the community
    let community_num_deleted = Community::delete(&mut conn, inserted_community.id).unwrap();
    assert_eq!(1, community_num_deleted);

    // Should be none found, since the creator was deleted
    let after_delete = PostAggregates::read(&mut conn, inserted_post.id);
    assert!(after_delete.is_err());
  }
}
