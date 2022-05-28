use crate::{aggregates::structs::SiteAggregates, schema::site_aggregates};
use diesel::{result::Error, *};

impl SiteAggregates {
  pub fn read(conn: &mut PgConnection) -> Result<Self, Error> {
    site_aggregates::table.first::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    aggregates::site_aggregates::SiteAggregates,
    source::{
      comment::{Comment, CommentForm},
      community::{Community, CommunityForm},
      person::{Person, PersonForm},
      post::{Post, PostForm},
      site::{Site, SiteForm},
    },
    traits::Crud,
    utils::establish_unpooled_connection,
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_person = PersonForm {
      name: "thommy_site_agg".into(),
      ..PersonForm::default()
    };

    let inserted_person = Person::create(&mut conn, &new_person).unwrap();

    let site_form = SiteForm {
      name: "test_site".into(),
      ..Default::default()
    };

    let inserted_site = Site::create(&mut conn, &site_form).unwrap();

    let new_community = CommunityForm {
      name: "TIL_site_agg".into(),
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

    // Insert two of those posts
    let inserted_post = Post::create(&mut conn, &new_post).unwrap();
    let _inserted_post_again = Post::create(&mut conn, &new_post).unwrap();

    let comment_form = CommentForm {
      content: "A test comment".into(),
      creator_id: inserted_person.id,
      post_id: inserted_post.id,
      ..CommentForm::default()
    };

    // Insert two of those comments
    let inserted_comment = Comment::create(&mut conn, &comment_form).unwrap();

    let child_comment_form = CommentForm {
      content: "A test comment".into(),
      creator_id: inserted_person.id,
      post_id: inserted_post.id,
      parent_id: Some(inserted_comment.id),
      ..CommentForm::default()
    };

    let _inserted_child_comment = Comment::create(&mut conn, &child_comment_form).unwrap();

    let site_aggregates_before_delete = SiteAggregates::read(&mut conn).unwrap();

    assert_eq!(1, site_aggregates_before_delete.users);
    assert_eq!(1, site_aggregates_before_delete.communities);
    assert_eq!(2, site_aggregates_before_delete.posts);
    assert_eq!(2, site_aggregates_before_delete.comments);

    // Try a post delete
    Post::delete(&mut conn, inserted_post.id).unwrap();
    let site_aggregates_after_post_delete = SiteAggregates::read(&mut conn).unwrap();
    assert_eq!(1, site_aggregates_after_post_delete.posts);
    assert_eq!(0, site_aggregates_after_post_delete.comments);

    // This shouuld delete all the associated rows, and fire triggers
    let person_num_deleted = Person::delete(&mut conn, inserted_person.id).unwrap();
    assert_eq!(1, person_num_deleted);

    // Delete the community
    let community_num_deleted = Community::delete(&mut conn, inserted_community.id).unwrap();
    assert_eq!(1, community_num_deleted);

    // Site should still exist, it can without a site creator.
    let after_delete_creator = SiteAggregates::read(&mut conn);
    assert!(after_delete_creator.is_ok());

    Site::delete(&mut conn, inserted_site.id).unwrap();
    let after_delete_site = SiteAggregates::read(&mut conn);
    assert!(after_delete_site.is_err());
  }
}
