use crate::{aggregates::structs::SiteAggregates, schema::site_aggregates, utils::DbConn};
use diesel::result::Error;
use diesel_async::RunQueryDsl;

impl SiteAggregates {
  pub async fn read(mut conn: impl DbConn) -> Result<Self, Error> {
    site_aggregates::table.first::<Self>(conn).await
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    aggregates::site_aggregates::SiteAggregates,
    source::{
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
      site::{Site, SiteInsertForm},
    },
    traits::Crud,
    utils::build_db_conn_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let mut conn = build_db_conn_for_tests().await;

    let inserted_instance = Instance::read_or_create(&mut *conn, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person = PersonInsertForm::builder()
      .name("thommy_site_agg".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(&mut *conn, &new_person).await.unwrap();

    let site_form = SiteInsertForm::builder()
      .name("test_site".into())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_site = Site::create(&mut *conn, &site_form).await.unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("TIL_site_agg".into())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(&mut *conn, &new_community).await.unwrap();

    let new_post = PostInsertForm::builder()
      .name("A test post".into())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .build();

    // Insert two of those posts
    let inserted_post = Post::create(&mut *conn, &new_post).await.unwrap();
    let _inserted_post_again = Post::create(&mut *conn, &new_post).await.unwrap();

    let comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    // Insert two of those comments
    let inserted_comment = Comment::create(&mut *conn, &comment_form, None)
      .await
      .unwrap();

    let child_comment_form = CommentInsertForm::builder()
      .content("A test comment".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    let _inserted_child_comment = Comment::create(
      &mut *conn,
      &child_comment_form,
      Some(&inserted_comment.path),
    )
    .await
    .unwrap();

    let site_aggregates_before_delete = SiteAggregates::read(&mut *conn).await.unwrap();

    // TODO: this is unstable, sometimes it returns 0 users, sometimes 1
    //assert_eq!(0, site_aggregates_before_delete.users);
    assert_eq!(1, site_aggregates_before_delete.communities);
    assert_eq!(2, site_aggregates_before_delete.posts);
    assert_eq!(2, site_aggregates_before_delete.comments);

    // Try a post delete
    Post::delete(&mut *conn, inserted_post.id).await.unwrap();
    let site_aggregates_after_post_delete = SiteAggregates::read(&mut *conn).await.unwrap();
    assert_eq!(1, site_aggregates_after_post_delete.posts);
    assert_eq!(0, site_aggregates_after_post_delete.comments);

    // This shouuld delete all the associated rows, and fire triggers
    let person_num_deleted = Person::delete(&mut *conn, inserted_person.id)
      .await
      .unwrap();
    assert_eq!(1, person_num_deleted);

    // Delete the community
    let community_num_deleted = Community::delete(&mut *conn, inserted_community.id)
      .await
      .unwrap();
    assert_eq!(1, community_num_deleted);

    // Site should still exist, it can without a site creator.
    let after_delete_creator = SiteAggregates::read(&mut *conn).await;
    assert!(after_delete_creator.is_ok());

    Site::delete(&mut *conn, inserted_site.id).await.unwrap();
    let after_delete_site = SiteAggregates::read(&mut *conn).await;
    assert!(after_delete_site.is_err());

    Instance::delete(&mut *conn, inserted_instance.id)
      .await
      .unwrap();
  }
}
