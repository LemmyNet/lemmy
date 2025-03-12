use crate::structs::{CommunityFollowerView, PendingFollow};
use chrono::Utc;
use diesel::{
  dsl::{count, count_star, exists, not},
  result::Error,
  select,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  impls::community::community_follower_select_subscribed_type,
  newtypes::{CommunityId, DbUrl, InstanceId, PersonId},
  schema::{community, community_actions, person},
  source::{
    community::{Community, CommunityFollowerState},
    person::Person,
  },
  utils::{get_conn, limit_and_offset, DbPool},
  CommunityVisibility,
  SubscribedType,
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl CommunityFollowerView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    community_actions::table
      .filter(community_actions::followed.is_not_null())
      .inner_join(community::table)
      .inner_join(person::table.on(community_actions::person_id.eq(person::id)))
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
      .filter(community_actions::followed.gt(published_since.naive_utc()))
      .select((community::id, person::inbox_url))
      .distinct() // only need each community_id, inbox combination once
      .load::<(CommunityId, DbUrl)>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn get_community_follower_inboxes(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
  ) -> Result<Vec<DbUrl>, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = Self::joins()
      .filter(community_actions::community_id.eq(community_id))
      .filter(not(person::local))
      .select(person::inbox_url)
      .distinct()
      .load::<DbUrl>(conn)
      .await?;

    Ok(res)
  }

  pub async fn count_community_followers(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
  ) -> Result<i64, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = Self::joins()
      .filter(community_actions::community_id.eq(community_id))
      .select(count_star())
      .first::<i64>(conn)
      .await?;

    Ok(res)
  }

  pub async fn for_person(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(community_actions::person_id.eq(person_id))
      .filter(community::deleted.eq(false))
      .filter(community::removed.eq(false))
      .select(Self::as_select())
      .order_by(community::title)
      .load::<CommunityFollowerView>(conn)
      .await
  }

  pub async fn list_approval_required(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    // TODO: if this is true dont check for community mod, but only check for local community
    //       also need to check is_admin()
    all_communities: bool,
    pending_only: bool,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<PendingFollow>, Error> {
    let conn = &mut get_conn(pool).await?;
    let (limit, offset) = limit_and_offset(page, limit)?;
    let (person_alias, community_follower_alias) = diesel::alias!(
      person as person_alias,
      community_actions as community_follower_alias
    );

    // check if the community already has an accepted follower from the same instance
    let is_new_instance = not(exists(
      person_alias
        .inner_join(
          community_follower_alias.on(
            person_alias
              .field(person::id)
              .eq(community_follower_alias.field(community_actions::person_id)),
          ),
        )
        .filter(
          person::instance_id
            .eq(person_alias.field(person::instance_id))
            .and(
              community_follower_alias
                .field(community_actions::community_id)
                .eq(community_actions::community_id),
            )
            .and(
              community_follower_alias
                .field(community_actions::follow_state)
                .eq(CommunityFollowerState::Accepted),
            ),
        ),
    ));

    let mut query = Self::joins()
      .select((
        person::all_columns,
        community::all_columns,
        is_new_instance,
        community_follower_select_subscribed_type(),
      ))
      .into_boxed();
    if all_communities {
      // if param is false, only return items for communities where user is a mod
      query = query
        .filter(community_actions::became_moderator.is_not_null())
        .filter(community_actions::person_id.eq(person_id));
    }
    if pending_only {
      query =
        query.filter(community_actions::follow_state.eq(CommunityFollowerState::ApprovalRequired));
    }
    let res = query
      .order_by(community_actions::followed.asc())
      .limit(limit)
      .offset(offset)
      .load::<(Person, Community, bool, SubscribedType)>(conn)
      .await?;
    Ok(
      res
        .into_iter()
        .map(
          |(person, community, is_new_instance, subscribed)| PendingFollow {
            person,
            community,
            is_new_instance,
            subscribed,
          },
        )
        .collect(),
    )
  }

  pub async fn count_approval_required(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
  ) -> Result<i64, Error> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(community_actions::community_id.eq(community_id))
      .filter(community_actions::follow_state.eq(CommunityFollowerState::ApprovalRequired))
      .select(count(community_actions::community_id))
      .first::<i64>(conn)
      .await
  }
  pub async fn check_private_community_action(
    pool: &mut DbPool<'_>,
    from_person_id: PersonId,
    community: &Community,
  ) -> LemmyResult<()> {
    if community.visibility != CommunityVisibility::Private {
      return Ok(());
    }
    let conn = &mut get_conn(pool).await?;
    select(exists(
      Self::joins()
        .filter(community_actions::community_id.eq(community.id))
        .filter(community_actions::person_id.eq(from_person_id))
        .filter(community_actions::follow_state.eq(CommunityFollowerState::Accepted)),
    ))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::NotFound.into())
  }
  pub async fn check_has_followers_from_instance(
    community_id: CommunityId,
    instance_id: InstanceId,
    pool: &mut DbPool<'_>,
  ) -> Result<(), Error> {
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
    .ok_or(diesel::NotFound)
  }

  pub async fn is_follower(
    community_id: CommunityId,
    instance_id: InstanceId,
    pool: &mut DbPool<'_>,
  ) -> Result<(), Error> {
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
    .ok_or(diesel::NotFound)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use lemmy_db_schema::{
    source::{
      community::{CommunityFollower, CommunityFollowerForm, CommunityInsertForm},
      instance::Instance,
      person::PersonInsertForm,
    },
    traits::{Crud, Followable},
    utils::build_db_pool_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_has_followers_from_instance() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    // insert local community
    let local_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;
    let community_form = CommunityInsertForm::new(
      local_instance.id,
      "test_community_3".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &community_form).await?;

    // insert remote user
    let remote_instance = Instance::read_or_create(pool, "other_domain.tld".to_string()).await?;
    let person_form =
      PersonInsertForm::new("name".to_string(), "pubkey".to_string(), remote_instance.id);
    let person = Person::create(pool, &person_form).await?;

    // community has no follower from remote instance, returns error
    let has_followers = CommunityFollowerView::check_has_followers_from_instance(
      community.id,
      remote_instance.id,
      pool,
    )
    .await;
    assert!(has_followers.is_err());

    // insert unapproved follower
    let mut follower_form = CommunityFollowerForm {
      state: Some(CommunityFollowerState::ApprovalRequired),
      ..CommunityFollowerForm::new(community.id, person.id)
    };
    CommunityFollower::follow(pool, &follower_form).await?;

    // still returns error
    let has_followers = CommunityFollowerView::check_has_followers_from_instance(
      community.id,
      remote_instance.id,
      pool,
    )
    .await;
    assert!(has_followers.is_err());

    // mark follower as accepted
    follower_form.state = Some(CommunityFollowerState::Accepted);
    CommunityFollower::follow(pool, &follower_form).await?;

    // now returns ok
    let has_followers = CommunityFollowerView::check_has_followers_from_instance(
      community.id,
      remote_instance.id,
      pool,
    )
    .await;
    assert!(has_followers.is_ok());

    Instance::delete(pool, local_instance.id).await?;
    Instance::delete(pool, remote_instance.id).await?;
    Ok(())
  }
}
