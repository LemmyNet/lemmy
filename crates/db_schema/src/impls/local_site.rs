use crate::{
  schema::local_site,
  source::local_site::{LocalSite, LocalSiteInsertForm, LocalSiteUpdateForm},
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error};
use diesel_async::RunQueryDsl;
use lemmy_utils::{build_cache, error::LemmyResult, CacheLock};
use std::sync::LazyLock;

impl LocalSite {
  pub async fn create(pool: &mut DbPool<'_>, form: &LocalSiteInsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(local_site::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn read(pool: &mut DbPool<'_>) -> LemmyResult<Self> {
    static CACHE: CacheLock<LocalSite> = LazyLock::new(build_cache);
    Ok(
      CACHE
        .try_get_with((), async {
          let conn = &mut get_conn(pool).await?;
          local_site::table.first(conn).await
        })
        .await?,
    )
  }
  pub async fn update(pool: &mut DbPool<'_>, form: &LocalSiteUpdateForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_site::table)
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn delete(pool: &mut DbPool<'_>) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(local_site::table).execute(conn).await
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::{
    source::{
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityInsertForm, CommunityUpdateForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
      site::{Site, SiteInsertForm},
    },
    traits::Crud,
    utils::{build_db_pool_for_tests, DbPool},
  };
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  async fn prepare_site_with_community(
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<(Instance, Person, Site, Community)> {
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "thommy_site_agg");

    let inserted_person = Person::create(pool, &new_person).await?;

    let site_form = SiteInsertForm::new("test_site".into(), inserted_instance.id);
    let inserted_site = Site::create(pool, &site_form).await?;

    let local_site_form = LocalSiteInsertForm::new(inserted_site.id);
    LocalSite::create(pool, &local_site_form).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "TIL_site_agg".into(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );

    let inserted_community = Community::create(pool, &new_community).await?;

    Ok((
      inserted_instance,
      inserted_person,
      inserted_site,
      inserted_community,
    ))
  }

  #[tokio::test]
  #[serial]
  async fn test_aggregates() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let (inserted_instance, inserted_person, inserted_site, inserted_community) =
      prepare_site_with_community(pool).await?;

    let new_post = PostInsertForm::new(
      "A test post".into(),
      inserted_person.id,
      inserted_community.id,
    );

    // Insert two of those posts
    let inserted_post = Post::create(pool, &new_post).await?;
    let _inserted_post_again = Post::create(pool, &new_post).await?;

    let comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );

    // Insert two of those comments
    let inserted_comment = Comment::create(pool, &comment_form, None).await?;

    let child_comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );
    let _inserted_child_comment =
      Comment::create(pool, &child_comment_form, Some(&inserted_comment.path)).await?;

    let site_aggregates_before_delete = LocalSite::read(pool).await?;

    // TODO: this is unstable, sometimes it returns 0 users, sometimes 1
    //assert_eq!(0, site_aggregates_before_delete.users);
    assert_eq!(1, site_aggregates_before_delete.communities);
    assert_eq!(2, site_aggregates_before_delete.posts);
    assert_eq!(2, site_aggregates_before_delete.comments);

    // Try a post delete
    Post::delete(pool, inserted_post.id).await?;
    let site_aggregates_after_post_delete = LocalSite::read(pool).await?;
    assert_eq!(1, site_aggregates_after_post_delete.posts);
    assert_eq!(0, site_aggregates_after_post_delete.comments);

    // This shouuld delete all the associated rows, and fire triggers
    let person_num_deleted = Person::delete(pool, inserted_person.id).await?;
    assert_eq!(1, person_num_deleted);

    // Delete the community
    let community_num_deleted = Community::delete(pool, inserted_community.id).await?;
    assert_eq!(1, community_num_deleted);

    // Site should still exist, it can without a site creator.
    let after_delete_creator = LocalSite::read(pool).await;
    assert!(after_delete_creator.is_ok());

    Site::delete(pool, inserted_site.id).await?;
    let after_delete_site = LocalSite::read(pool).await;
    assert!(after_delete_site.is_err());

    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_soft_delete() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let (inserted_instance, inserted_person, inserted_site, inserted_community) =
      prepare_site_with_community(pool).await?;

    let site_aggregates_before = LocalSite::read(pool).await?;
    assert_eq!(1, site_aggregates_before.communities);

    Community::update(
      pool,
      inserted_community.id,
      &CommunityUpdateForm {
        deleted: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let site_aggregates_after_delete = LocalSite::read(pool).await?;
    assert_eq!(0, site_aggregates_after_delete.communities);

    Community::update(
      pool,
      inserted_community.id,
      &CommunityUpdateForm {
        deleted: Some(false),
        ..Default::default()
      },
    )
    .await?;

    Community::update(
      pool,
      inserted_community.id,
      &CommunityUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let site_aggregates_after_remove = LocalSite::read(pool).await?;
    assert_eq!(0, site_aggregates_after_remove.communities);

    Community::update(
      pool,
      inserted_community.id,
      &CommunityUpdateForm {
        deleted: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let site_aggregates_after_remove_delete = LocalSite::read(pool).await?;
    assert_eq!(0, site_aggregates_after_remove_delete.communities);

    Community::delete(pool, inserted_community.id).await?;
    Site::delete(pool, inserted_site.id).await?;
    Person::delete(pool, inserted_person.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }
}
