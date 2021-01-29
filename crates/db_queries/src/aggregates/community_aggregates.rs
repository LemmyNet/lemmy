use diesel::{result::Error, *};
use lemmy_db_schema::schema::community_aggregates;
use serde::Serialize;

#[derive(Queryable, Associations, Identifiable, PartialEq, Debug, Serialize, Clone)]
#[table_name = "community_aggregates"]
pub struct CommunityAggregates {
  pub id: i32,
  pub community_id: i32,
  pub subscribers: i64,
  pub posts: i64,
  pub comments: i64,
  pub published: chrono::NaiveDateTime,
  pub users_active_day: i64,
  pub users_active_week: i64,
  pub users_active_month: i64,
  pub users_active_half_year: i64,
}

impl CommunityAggregates {
  pub fn read(conn: &PgConnection, community_id: i32) -> Result<Self, Error> {
    community_aggregates::table
      .filter(community_aggregates::community_id.eq(community_id))
      .first::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    aggregates::community_aggregates::CommunityAggregates,
    establish_unpooled_connection,
    Crud,
    Followable,
    ListingType,
    SortType,
  };
  use lemmy_db_schema::source::{
    comment::{Comment, CommentForm},
    community::{Community, CommunityFollower, CommunityFollowerForm, CommunityForm},
    post::{Post, PostForm},
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

    let another_community = CommunityForm {
      name: "TIL_community_agg_2".into(),
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

    let another_inserted_community = Community::create(&conn, &another_community).unwrap();

    let first_user_follow = CommunityFollowerForm {
      community_id: inserted_community.id,
      user_id: inserted_user.id,
      pending: false,
    };

    CommunityFollower::follow(&conn, &first_user_follow).unwrap();

    let second_user_follow = CommunityFollowerForm {
      community_id: inserted_community.id,
      user_id: another_inserted_user.id,
      pending: false,
    };

    CommunityFollower::follow(&conn, &second_user_follow).unwrap();

    let another_community_follow = CommunityFollowerForm {
      community_id: another_inserted_community.id,
      user_id: inserted_user.id,
      pending: false,
    };

    CommunityFollower::follow(&conn, &another_community_follow).unwrap();

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

    let community_aggregates_before_delete =
      CommunityAggregates::read(&conn, inserted_community.id).unwrap();

    assert_eq!(2, community_aggregates_before_delete.subscribers);
    assert_eq!(1, community_aggregates_before_delete.posts);
    assert_eq!(2, community_aggregates_before_delete.comments);

    // Test the other community
    let another_community_aggs =
      CommunityAggregates::read(&conn, another_inserted_community.id).unwrap();
    assert_eq!(1, another_community_aggs.subscribers);
    assert_eq!(0, another_community_aggs.posts);
    assert_eq!(0, another_community_aggs.comments);

    // Unfollow test
    CommunityFollower::unfollow(&conn, &second_user_follow).unwrap();
    let after_unfollow = CommunityAggregates::read(&conn, inserted_community.id).unwrap();
    assert_eq!(1, after_unfollow.subscribers);

    // Follow again just for the later tests
    CommunityFollower::follow(&conn, &second_user_follow).unwrap();
    let after_follow_again = CommunityAggregates::read(&conn, inserted_community.id).unwrap();
    assert_eq!(2, after_follow_again.subscribers);

    // Remove a parent comment (the comment count should also be 0)
    Post::delete(&conn, inserted_post.id).unwrap();
    let after_parent_post_delete = CommunityAggregates::read(&conn, inserted_community.id).unwrap();
    assert_eq!(0, after_parent_post_delete.comments);
    assert_eq!(0, after_parent_post_delete.posts);

    // Remove the 2nd user
    User_::delete(&conn, another_inserted_user.id).unwrap();
    let after_user_delete = CommunityAggregates::read(&conn, inserted_community.id).unwrap();
    assert_eq!(1, after_user_delete.subscribers);

    // This should delete all the associated rows, and fire triggers
    let user_num_deleted = User_::delete(&conn, inserted_user.id).unwrap();
    assert_eq!(1, user_num_deleted);

    // Should be none found, since the creator was deleted
    let after_delete = CommunityAggregates::read(&conn, inserted_community.id);
    assert!(after_delete.is_err());
  }
}
