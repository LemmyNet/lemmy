use crate::{
  source::local_site::{LocalSite, LocalSiteInsertForm, LocalSiteUpdateForm},
  utils::{DbPool, get_conn},
};
use diesel::dsl::insert_into;
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::local_site;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl LocalSite {
  pub async fn create(pool: &mut DbPool<'_>, form: &LocalSiteInsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(local_site::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  pub async fn update(pool: &mut DbPool<'_>, form: &LocalSiteUpdateForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_site::table)
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  pub async fn delete(pool: &mut DbPool<'_>) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(local_site::table)
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::Deleted)
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
      site::Site,
    },
    test_data::TestData,
    traits::Crud,
    utils::{DbPool, build_db_pool_for_tests},
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  async fn read_local_site(pool: &mut DbPool<'_>) -> LemmyResult<LocalSite> {
    let conn = &mut get_conn(pool).await?;
    local_site::table
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  async fn prepare_site_with_community(
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<(TestData, Person, Community)> {
    let data = TestData::create(pool).await?;

    let new_person = PersonInsertForm::test_form(data.instance.id, "thommy_site_agg");
    let inserted_person = Person::create(pool, &new_person).await?;

    let new_community = CommunityInsertForm::new(
      data.instance.id,
      "TIL_site_agg".into(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );

    let inserted_community = Community::create(pool, &new_community).await?;

    Ok((data, inserted_person, inserted_community))
  }

  #[tokio::test]
  #[serial]
  async fn test_aggregates() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let (data, inserted_person, inserted_community) = prepare_site_with_community(pool).await?;

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

    let site_aggregates_before_delete = read_local_site(pool).await?;

    // TODO: this is unstable, sometimes it returns 0 users, sometimes 1
    //assert_eq!(0, site_aggregates_before_delete.users);
    assert_eq!(1, site_aggregates_before_delete.communities);
    assert_eq!(2, site_aggregates_before_delete.posts);
    assert_eq!(2, site_aggregates_before_delete.comments);

    // Try a post delete
    Post::delete(pool, inserted_post.id).await?;
    let site_aggregates_after_post_delete = read_local_site(pool).await?;
    assert_eq!(1, site_aggregates_after_post_delete.posts);
    assert_eq!(0, site_aggregates_after_post_delete.comments);

    // This shouuld delete all the associated rows, and fire triggers
    let person_num_deleted = Person::delete(pool, inserted_person.id).await?;
    assert_eq!(1, person_num_deleted);

    // Delete the community
    let community_num_deleted = Community::delete(pool, inserted_community.id).await?;
    assert_eq!(1, community_num_deleted);

    // Site should still exist, it can without a site creator.
    let after_delete_creator = read_local_site(pool).await;
    assert!(after_delete_creator.is_ok());

    Site::delete(pool, data.site.id).await?;
    let after_delete_site = read_local_site(pool).await;
    assert!(after_delete_site.is_err());

    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_soft_delete() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let (data, inserted_person, inserted_community) = prepare_site_with_community(pool).await?;

    let site_aggregates_before = read_local_site(pool).await?;
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

    let site_aggregates_after_delete = read_local_site(pool).await?;
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

    let site_aggregates_after_remove = read_local_site(pool).await?;
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

    let site_aggregates_after_remove_delete = read_local_site(pool).await?;
    assert_eq!(0, site_aggregates_after_remove_delete.communities);

    Community::delete(pool, inserted_community.id).await?;
    Person::delete(pool, inserted_person.id).await?;
    data.delete(pool).await?;

    Ok(())
  }
}
