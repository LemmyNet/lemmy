use diesel::{result::Error, *};
use lemmy_db_schema::schema::site_aggregates;
use serde::Serialize;

#[derive(Queryable, Associations, Identifiable, PartialEq, Debug, Serialize, Clone)]
#[table_name = "site_aggregates"]
pub struct SiteAggregates {
  pub id: i32,
  pub site_id: i32,
  pub users: i64,
  pub posts: i64,
  pub comments: i64,
  pub communities: i64,
  pub users_active_day: i64,
  pub users_active_week: i64,
  pub users_active_month: i64,
  pub users_active_half_year: i64,
}

impl SiteAggregates {
  pub fn read(conn: &PgConnection) -> Result<Self, Error> {
    site_aggregates::table.first::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    aggregates::site_aggregates::SiteAggregates,
    establish_unpooled_connection,
    Crud,
    ListingType,
    SortType,
  };
  use lemmy_db_schema::source::{
    comment::{Comment, CommentForm},
    community::{Community, CommunityForm},
    post::{Post, PostForm},
    site::{Site, SiteForm},
    user::{UserForm, User_},
  };

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_user = UserForm {
      name: "thommy_site_agg".into(),
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

    let site_form = SiteForm {
      name: "test_site".into(),
      description: None,
      icon: None,
      banner: None,
      creator_id: inserted_user.id,
      enable_downvotes: true,
      open_registration: true,
      enable_nsfw: true,
      updated: None,
    };

    Site::create(&conn, &site_form).unwrap();

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

    // Insert two of those posts
    let inserted_post = Post::create(&conn, &new_post).unwrap();
    let _inserted_post_again = Post::create(&conn, &new_post).unwrap();

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

    // Insert two of those comments
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

    let site_aggregates_before_delete = SiteAggregates::read(&conn).unwrap();

    assert_eq!(1, site_aggregates_before_delete.users);
    assert_eq!(1, site_aggregates_before_delete.communities);
    assert_eq!(2, site_aggregates_before_delete.posts);
    assert_eq!(2, site_aggregates_before_delete.comments);

    // Try a post delete
    Post::delete(&conn, inserted_post.id).unwrap();
    let site_aggregates_after_post_delete = SiteAggregates::read(&conn).unwrap();
    assert_eq!(1, site_aggregates_after_post_delete.posts);
    assert_eq!(0, site_aggregates_after_post_delete.comments);

    // This shouuld delete all the associated rows, and fire triggers
    let user_num_deleted = User_::delete(&conn, inserted_user.id).unwrap();
    assert_eq!(1, user_num_deleted);

    let after_delete = SiteAggregates::read(&conn);
    assert!(after_delete.is_err());
  }
}
