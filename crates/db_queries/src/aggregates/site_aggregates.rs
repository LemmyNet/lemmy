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
  };
  use lemmy_db_schema::source::{
    comment::{Comment, CommentForm},
    community::{Community, CommunityForm},
    post::{Post, PostForm},
    site::{Site, SiteForm},
    person::{PersonForm, Person},
  };

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_person = PersonForm {
      name: "thommy_site_agg".into(),
      preferred_username: None,
      avatar: None,
      banner: None,
      banned: Some(false),
      deleted: false,
      published: None,
      updated: None,
      actor_id: None,
      bio: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
      inbox_url: None,
      shared_inbox_url: None,
    };

    let inserted_person = Person::create(&conn, &new_person).unwrap();

    let site_form = SiteForm {
      name: "test_site".into(),
      description: None,
      icon: None,
      banner: None,
      creator_id: inserted_person.id,
      enable_downvotes: true,
      open_registration: true,
      enable_nsfw: true,
      updated: None,
    };

    Site::create(&conn, &site_form).unwrap();

    let new_community = CommunityForm {
      name: "TIL_site_agg".into(),
      creator_id: inserted_person.id,
      title: "nada".to_owned(),
      description: None,
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
      followers_url: None,
      inbox_url: None,
      shared_inbox_url: None,
    };

    let inserted_community = Community::create(&conn, &new_community).unwrap();

    let new_post = PostForm {
      name: "A test post".into(),
      url: None,
      body: None,
      creator_id: inserted_person.id,
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
      creator_id: inserted_person.id,
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
      creator_id: inserted_person.id,
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
    let person_num_deleted = Person::delete(&conn, inserted_person.id).unwrap();
    assert_eq!(1, person_num_deleted);

    let after_delete = SiteAggregates::read(&conn);
    assert!(after_delete.is_err());
  }
}
