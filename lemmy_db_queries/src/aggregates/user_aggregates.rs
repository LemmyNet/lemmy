use diesel::{result::Error, *};
use lemmy_db_schema::schema::user_aggregates;
use serde::Serialize;

#[derive(Queryable, Associations, Identifiable, PartialEq, Debug, Serialize, Clone)]
#[table_name = "user_aggregates"]
pub struct UserAggregates {
  pub id: i32,
  pub user_id: i32,
  pub post_count: i64,
  pub post_score: i64,
  pub comment_count: i64,
  pub comment_score: i64,
}

impl UserAggregates {
  pub fn read(conn: &PgConnection, user_id: i32) -> Result<Self, Error> {
    user_aggregates::table
      .filter(user_aggregates::user_id.eq(user_id))
      .first::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    aggregates::user_aggregates::UserAggregates,
    establish_unpooled_connection,
    Crud,
    Likeable,
    ListingType,
    SortType,
  };
  use lemmy_db_schema::source::{
    comment::{Comment, CommentForm, CommentLike, CommentLikeForm},
    community::{Community, CommunityForm},
    post::{Post, PostForm, PostLike, PostLikeForm},
    user::{UserForm, User_},
  };

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_user = UserForm {
      name: "thommy_user_agg".into(),
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
      name: "jerry_user_agg".into(),
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
      name: "TIL_site_agg".into(),
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

    let post_like = PostLikeForm {
      post_id: inserted_post.id,
      user_id: inserted_user.id,
      score: 1,
    };

    let _inserted_post_like = PostLike::like(&conn, &post_like).unwrap();

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

    let mut comment_like = CommentLikeForm {
      comment_id: inserted_comment.id,
      user_id: inserted_user.id,
      post_id: inserted_post.id,
      score: 1,
    };

    let _inserted_comment_like = CommentLike::like(&conn, &comment_like).unwrap();

    let mut child_comment_form = CommentForm {
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

    let inserted_child_comment = Comment::create(&conn, &child_comment_form).unwrap();

    let child_comment_like = CommentLikeForm {
      comment_id: inserted_child_comment.id,
      user_id: another_inserted_user.id,
      post_id: inserted_post.id,
      score: 1,
    };

    let _inserted_child_comment_like = CommentLike::like(&conn, &child_comment_like).unwrap();

    let user_aggregates_before_delete = UserAggregates::read(&conn, inserted_user.id).unwrap();

    assert_eq!(1, user_aggregates_before_delete.post_count);
    assert_eq!(1, user_aggregates_before_delete.post_score);
    assert_eq!(2, user_aggregates_before_delete.comment_count);
    assert_eq!(2, user_aggregates_before_delete.comment_score);

    // Remove a post like
    PostLike::remove(&conn, inserted_user.id, inserted_post.id).unwrap();
    let after_post_like_remove = UserAggregates::read(&conn, inserted_user.id).unwrap();
    assert_eq!(0, after_post_like_remove.post_score);

    // Remove a parent comment (the scores should also be removed)
    Comment::delete(&conn, inserted_comment.id).unwrap();
    let after_parent_comment_delete = UserAggregates::read(&conn, inserted_user.id).unwrap();
    assert_eq!(0, after_parent_comment_delete.comment_count);
    assert_eq!(0, after_parent_comment_delete.comment_score);

    // Add in the two comments again, then delete the post.
    let new_parent_comment = Comment::create(&conn, &comment_form).unwrap();
    child_comment_form.parent_id = Some(new_parent_comment.id);
    Comment::create(&conn, &child_comment_form).unwrap();
    comment_like.comment_id = new_parent_comment.id;
    CommentLike::like(&conn, &comment_like).unwrap();
    let after_comment_add = UserAggregates::read(&conn, inserted_user.id).unwrap();
    assert_eq!(2, after_comment_add.comment_count);
    assert_eq!(1, after_comment_add.comment_score);

    Post::delete(&conn, inserted_post.id).unwrap();
    let after_post_delete = UserAggregates::read(&conn, inserted_user.id).unwrap();
    assert_eq!(0, after_post_delete.comment_score);
    assert_eq!(0, after_post_delete.comment_count);
    assert_eq!(0, after_post_delete.post_score);
    assert_eq!(0, after_post_delete.post_count);

    // This should delete all the associated rows, and fire triggers
    let user_num_deleted = User_::delete(&conn, inserted_user.id).unwrap();
    assert_eq!(1, user_num_deleted);
    User_::delete(&conn, another_inserted_user.id).unwrap();

    // Should be none found
    let after_delete = UserAggregates::read(&conn, inserted_user.id);
    assert!(after_delete.is_err());
  }
}
