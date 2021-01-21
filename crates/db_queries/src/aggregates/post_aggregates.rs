use diesel::{result::Error, *};
use lemmy_db_schema::schema::post_aggregates;
use serde::Serialize;

#[derive(Queryable, Associations, Identifiable, PartialEq, Debug, Serialize, Clone)]
#[table_name = "post_aggregates"]
pub struct PostAggregates {
  pub id: i32,
  pub post_id: i32,
  pub comments: i64,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub stickied: bool,
  pub published: chrono::NaiveDateTime,
  pub newest_comment_time: chrono::NaiveDateTime,
}

impl PostAggregates {
  pub fn read(conn: &PgConnection, post_id: i32) -> Result<Self, Error> {
    post_aggregates::table
      .filter(post_aggregates::post_id.eq(post_id))
      .first::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    aggregates::post_aggregates::PostAggregates,
    establish_unpooled_connection,
    Crud,
    Likeable,
    ListingType,
    SortType,
  };
  use lemmy_db_schema::source::{
    comment::{Comment, CommentForm},
    community::{Community, CommunityForm},
    post::{Post, PostForm, PostLike, PostLikeForm},
    user::{UserForm, User_},
  };

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_user = UserForm {
      name: "thommy_community_agg".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      matrix_user_id: None,
      avatar: None,
      banner: None,
      admin: false,
      banned: Some(false),
      published: None,
      updated: None,
      show_nsfw: false,
      theme: "browser".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
      actor_id: None,
      bio: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
    };

    let inserted_user = User_::create(&conn, &new_user).unwrap();

    let another_user = UserForm {
      name: "jerry_community_agg".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      matrix_user_id: None,
      avatar: None,
      banner: None,
      admin: false,
      banned: Some(false),
      published: None,
      updated: None,
      show_nsfw: false,
      theme: "browser".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
      actor_id: None,
      bio: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
    };

    let another_inserted_user = User_::create(&conn, &another_user).unwrap();

    let new_community = CommunityForm {
      name: "TIL_community_agg".into(),
      creator_id: inserted_user.id,
      title: "nada".to_owned(),
      description: None,
      category_id: 1,
      nsfw: false,
      removed: None,
      deleted: None,
      updated: None,
      actor_id: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
      published: None,
      icon: None,
      banner: None,
    };

    let inserted_community = Community::create(&conn, &new_community).unwrap();

    let new_post = PostForm {
      name: "A test post".into(),
      url: None,
      body: None,
      creator_id: inserted_user.id,
      community_id: inserted_community.id,
      removed: None,
      deleted: None,
      locked: None,
      stickied: None,
      nsfw: false,
      updated: None,
      embed_title: None,
      embed_description: None,
      embed_html: None,
      thumbnail_url: None,
      ap_id: None,
      local: true,
      published: None,
    };

    let inserted_post = Post::create(&conn, &new_post).unwrap();

    let comment_form = CommentForm {
      content: "A test comment".into(),
      creator_id: inserted_user.id,
      post_id: inserted_post.id,
      removed: None,
      deleted: None,
      read: None,
      parent_id: None,
      published: None,
      updated: None,
      ap_id: None,
      local: true,
    };

    let inserted_comment = Comment::create(&conn, &comment_form).unwrap();

    let child_comment_form = CommentForm {
      content: "A test comment".into(),
      creator_id: inserted_user.id,
      post_id: inserted_post.id,
      removed: None,
      deleted: None,
      read: None,
      parent_id: Some(inserted_comment.id),
      published: None,
      updated: None,
      ap_id: None,
      local: true,
    };

    let _inserted_child_comment = Comment::create(&conn, &child_comment_form).unwrap();

    let post_like = PostLikeForm {
      post_id: inserted_post.id,
      user_id: inserted_user.id,
      score: 1,
    };

    PostLike::like(&conn, &post_like).unwrap();

    let post_aggs_before_delete = PostAggregates::read(&conn, inserted_post.id).unwrap();

    assert_eq!(2, post_aggs_before_delete.comments);
    assert_eq!(1, post_aggs_before_delete.score);
    assert_eq!(1, post_aggs_before_delete.upvotes);
    assert_eq!(0, post_aggs_before_delete.downvotes);

    // Add a post dislike from the other user
    let post_dislike = PostLikeForm {
      post_id: inserted_post.id,
      user_id: another_inserted_user.id,
      score: -1,
    };

    PostLike::like(&conn, &post_dislike).unwrap();

    let post_aggs_after_dislike = PostAggregates::read(&conn, inserted_post.id).unwrap();

    assert_eq!(2, post_aggs_after_dislike.comments);
    assert_eq!(0, post_aggs_after_dislike.score);
    assert_eq!(1, post_aggs_after_dislike.upvotes);
    assert_eq!(1, post_aggs_after_dislike.downvotes);

    // Remove the parent comment
    Comment::delete(&conn, inserted_comment.id).unwrap();
    let after_comment_delete = PostAggregates::read(&conn, inserted_post.id).unwrap();
    assert_eq!(0, after_comment_delete.comments);
    assert_eq!(0, after_comment_delete.score);
    assert_eq!(1, after_comment_delete.upvotes);
    assert_eq!(1, after_comment_delete.downvotes);

    // Remove the first post like
    PostLike::remove(&conn, inserted_user.id, inserted_post.id).unwrap();
    let after_like_remove = PostAggregates::read(&conn, inserted_post.id).unwrap();
    assert_eq!(0, after_like_remove.comments);
    assert_eq!(-1, after_like_remove.score);
    assert_eq!(0, after_like_remove.upvotes);
    assert_eq!(1, after_like_remove.downvotes);

    // This should delete all the associated rows, and fire triggers
    User_::delete(&conn, another_inserted_user.id).unwrap();
    let user_num_deleted = User_::delete(&conn, inserted_user.id).unwrap();
    assert_eq!(1, user_num_deleted);

    // Should be none found, since the creator was deleted
    let after_delete = PostAggregates::read(&conn, inserted_post.id);
    assert!(after_delete.is_err());
  }
}
