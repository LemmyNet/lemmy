use crate::CommunityFollowerView;
use chrono::Utc;
use diesel::{
  ExpressionMethods,
  JoinOnDsl,
  QueryDsl,
  SelectableHelper,
  dsl::{count_star, exists, not},
  select,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::utils::queries::filters::filter_is_subscribed;
use lemmy_db_schema_file::{
  InstanceId,
  PersonId,
  enums::CommunityFollowerState,
  newtypes::CommunityId,
  schema::{community, community_actions, person},
};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  dburl::DbUrl,
  utils::functions::lower,
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl CommunityFollowerView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    community_actions::table
      .inner_join(community::table)
      .inner_join(person::table.on(community_actions::person_id.eq(person::id)))
      .filter(community_actions::followed_at.is_not_null())
  }
  /// return a list of local community ids and remote inboxes that at least one user of the given
  /// instance has followed
  pub async fn get_instance_followed_community_inboxes(
    pool: &mut DbPool<'_>,
    instance_id: InstanceId,
    published_since: chrono::DateTime<Utc>,
  ) -> LemmyResult<Vec<(CommunityId, DbUrl)>> {
    let conn = &mut get_conn(pool).await?;
    // In most cases this will fetch the same url many times (the shared inbox url)
    // PG will only send a single copy to rust, but it has to scan through all follower rows (same
    // as it was before). So on the PG side it would be possible to optimize this further by
    // adding e.g. a new table community_followed_instances (community_id, instance_id)
    // that would work for all instances that support fully shared inboxes.
    // It would be a bit more complicated though to keep it in sync.

    Self::joins()
      .filter(person::instance_id.eq(instance_id))
      .filter(community::local) // this should be a no-op since community_followers table only has
      // local-person+remote-community or remote-person+local-community
      .filter(not(person::local))
      .filter(community_actions::followed_at.gt(published_since.naive_utc()))
      .filter(filter_is_subscribed())
      .select((community::id, person::inbox_url))
      .distinct() // only need each community_id, inbox combination once
      .load::<(CommunityId, DbUrl)>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn count_community_followers(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
  ) -> LemmyResult<i32> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(community_actions::community_id.eq(community_id))
      .filter(filter_is_subscribed())
      .select(count_star())
      .first::<i64>(conn)
      .await
      .map(i32::try_from)?
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn for_person(pool: &mut DbPool<'_>, person_id: PersonId) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(community_actions::person_id.eq(person_id))
      .filter(community::deleted.eq(false))
      .filter(community::removed.eq(false))
      .filter(community::local_removed.eq(false))
      .filter(filter_is_subscribed())
      .select(Self::as_select())
      .order_by(lower(community::name))
      .load::<CommunityFollowerView>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn is_follower(
    community_id: CommunityId,
    instance_id: InstanceId,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      Self::joins()
        .filter(community_actions::community_id.eq(community_id))
        .filter(person::instance_id.eq(instance_id))
        .filter(community_actions::follow_state.eq(CommunityFollowerState::Accepted)),
    ))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::NotFound.into())
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {
  use super::*;
  use chrono::TimeDelta;
  use lemmy_db_schema::{
    source::{
      community::{Community, CommunityActions, CommunityFollowerForm, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
    },
    traits::Followable,
  };
  use lemmy_db_schema_file::enums::CommunityVisibility;
  use lemmy_diesel_utils::{connection::build_db_pool_for_tests, dburl::DbUrl, traits::Crud};
  use pretty_assertions::assert_eq;
  use serial_test::serial;
  use url::Url;

  #[tokio::test]
  #[serial]
  async fn test_follower_view_functions() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    // local private community
    let local_instance = Instance::read_or_create(pool, "my_domain.tld").await?;
    let community_form = CommunityInsertForm {
      visibility: Some(CommunityVisibility::Private),
      ..CommunityInsertForm::new(
        local_instance.id,
        "test_community".to_string(),
        "pubkey".to_string(),
      )
    };
    let community = Community::create(pool, &community_form).await?;

    // remote user with a known inbox url
    let remote_instance = Instance::read_or_create(pool, "other_domain.tld").await?;
    let remote_inbox = DbUrl(Url::parse("https://other_domain.tld/inbox")?.into());
    let remote_person = Person::create(
      pool,
      &PersonInsertForm {
        inbox_url: Some(remote_inbox.clone()),
        local: Some(false),
        ..PersonInsertForm::new(
          "remote_user".to_string(),
          "pubkey".to_string(),
          remote_instance.id,
        )
      },
    )
    .await?;

    let mut follower_form = CommunityFollowerForm::new(
      community.id,
      remote_person.id,
      CommunityFollowerState::ApprovalRequired,
    );
    CommunityActions::follow(pool, &follower_form).await?;

    // pending follow excluded
    let inboxes = CommunityFollowerView::get_instance_followed_community_inboxes(
      pool,
      remote_instance.id,
      Utc::now() - TimeDelta::days(1),
    )
    .await?;
    assert!(inboxes.is_empty());
    let follower_count =
      CommunityFollowerView::count_community_followers(pool, community.id).await?;
    assert_eq!(0, follower_count);
    let followers = CommunityFollowerView::for_person(pool, remote_person.id).await?;
    assert!(followers.is_empty());
    assert!(
      CommunityFollowerView::is_follower(community.id, remote_instance.id, pool)
        .await
        .is_err()
    );

    // denied follow also excluded
    follower_form.follow_state = CommunityFollowerState::Denied;
    CommunityActions::follow(pool, &follower_form).await?;
    let inboxes = CommunityFollowerView::get_instance_followed_community_inboxes(
      pool,
      remote_instance.id,
      Utc::now() - TimeDelta::days(1),
    )
    .await?;
    assert!(inboxes.is_empty());
    let follower_count =
      CommunityFollowerView::count_community_followers(pool, community.id).await?;
    assert_eq!(0, follower_count);
    let followers = CommunityFollowerView::for_person(pool, remote_person.id).await?;
    assert!(followers.is_empty());
    assert!(
      CommunityFollowerView::is_follower(community.id, remote_instance.id, pool)
        .await
        .is_err()
    );

    // accepted follow is returned
    follower_form.follow_state = CommunityFollowerState::Accepted;
    CommunityActions::follow(pool, &follower_form).await?;
    let inboxes = CommunityFollowerView::get_instance_followed_community_inboxes(
      pool,
      remote_instance.id,
      Utc::now() - TimeDelta::days(1),
    )
    .await?;
    assert_eq!(1, inboxes.len());
    assert_eq!(inboxes[0].0, community.id);
    assert_eq!(inboxes[0].1, remote_inbox);
    let follower_count =
      CommunityFollowerView::count_community_followers(pool, community.id).await?;
    assert_eq!(1, follower_count);
    let followers = CommunityFollowerView::for_person(pool, remote_person.id).await?;
    assert_eq!(1, followers.len());
    assert_eq!(followers[0].follower.id, remote_person.id);
    assert!(
      CommunityFollowerView::is_follower(community.id, remote_instance.id, pool)
        .await
        .is_ok()
    );

    // follow newer than published_since is not returned
    let inboxes = CommunityFollowerView::get_instance_followed_community_inboxes(
      pool,
      remote_instance.id,
      Utc::now() + TimeDelta::days(1),
    )
    .await?;
    assert!(inboxes.is_empty());

    // cleanup
    Instance::delete(pool, local_instance.id).await?;
    Instance::delete(pool, remote_instance.id).await?;
    Ok(())
  }
}
