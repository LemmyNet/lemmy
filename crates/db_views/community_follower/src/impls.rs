use crate::{CommunityFollowerView, PendingFollow};
use chrono::Utc;
use diesel::{
  dsl::{count, count_star, exists, not},
  select,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::{
  newtypes::{CommunityId, DbUrl, InstanceId, PaginationCursor, PersonId},
  source::{
    community::{community_actions_keys as key, Community, CommunityActions},
    person::Person,
  },
  traits::PaginationCursorBuilder,
  utils::{get_conn, limit_fetch, paginate, DbPool},
};
use lemmy_db_schema_file::{
  enums::{CommunityFollowerState, CommunityVisibility},
  schema::{community, community_actions, person},
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl CommunityFollowerView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    community_actions::table
      .filter(community_actions::followed_at.is_not_null())
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
      .filter(community_actions::followed_at.gt(published_since.naive_utc()))
      .select((community::id, person::inbox_url))
      .distinct() // only need each community_id, inbox combination once
      .load::<(CommunityId, DbUrl)>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn get_community_follower_inboxes(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
  ) -> LemmyResult<Vec<DbUrl>> {
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
  ) -> LemmyResult<i64> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(community_actions::community_id.eq(community_id))
      .select(count_star())
      .first::<i64>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn for_person(pool: &mut DbPool<'_>, person_id: PersonId) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(community_actions::person_id.eq(person_id))
      .filter(community::deleted.eq(false))
      .filter(community::removed.eq(false))
      .filter(community::local_removed.eq(false))
      .select(Self::as_select())
      .order_by(community::title)
      .load::<CommunityFollowerView>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn list_approval_required(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    // TODO: if this is true dont check for community mod, but only check for local community
    //       also need to check is_admin()
    all_communities: bool,
    pending_only: bool,
    cursor_data: Option<CommunityActions>,
    page_back: Option<bool>,
    limit: Option<i64>,
  ) -> LemmyResult<Vec<PendingFollow>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(limit)?;
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
        community_actions::follow_state.nullable(),
      ))
      .limit(limit)
      .into_boxed();
    if all_communities {
      // if param is false, only return items for communities where user is a mod
      query = query
        .filter(community_actions::became_moderator_at.is_not_null())
        .filter(community_actions::person_id.eq(person_id));
    }
    if pending_only {
      query =
        query.filter(community_actions::follow_state.eq(CommunityFollowerState::ApprovalRequired));
    }

    // Sorting by published
    let paginated_query = paginate(query, SortDirection::Asc, cursor_data, None, page_back)
      .then_order_by(key::followed_at);

    let res = paginated_query
      .load::<(Person, Community, bool, Option<CommunityFollowerState>)>(conn)
      .await?;
    Ok(
      res
        .into_iter()
        .map(
          |(person, community, is_new_instance, follow_state)| PendingFollow {
            person,
            community,
            is_new_instance,
            follow_state,
          },
        )
        .collect(),
    )
  }

  pub async fn count_approval_required(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
  ) -> LemmyResult<i64> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(community_actions::community_id.eq(community_id))
      .filter(community_actions::follow_state.eq(CommunityFollowerState::ApprovalRequired))
      .select(count(community_actions::community_id))
      .first::<i64>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
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

impl PaginationCursorBuilder for CommunityFollowerView {
  type CursorData = CommunityActions;

  fn to_cursor(&self) -> PaginationCursor {
    // This needs a person and community
    let prefixes_and_ids = [('P', self.follower.id.0), ('C', self.community.id.0)];

    PaginationCursor::new(&prefixes_and_ids)
  }
  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::CursorData> {
    let pids = cursor.prefixes_and_ids();
    let (_, person_id) = pids
      .as_slice()
      .first()
      .ok_or(LemmyErrorType::CouldntParsePaginationToken)?;
    let (_, community_id) = pids
      .get(1)
      .ok_or(LemmyErrorType::CouldntParsePaginationToken)?;
    CommunityActions::read(pool, CommunityId(*community_id), PersonId(*person_id)).await
  }
}

impl PendingFollow {
  pub fn to_cursor(&self) -> PaginationCursor {
    // Build a fake community_follower_view to use its pagination cursor.
    let cfv = CommunityFollowerView {
      community: self.community.clone(),
      follower: self.person.clone(),
    };
    cfv.to_cursor()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use lemmy_db_schema::{
    source::{
      community::{CommunityActions, CommunityFollowerForm, CommunityInsertForm},
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
    let mut follower_form = CommunityFollowerForm::new(
      community.id,
      person.id,
      CommunityFollowerState::ApprovalRequired,
    );
    CommunityActions::follow(pool, &follower_form).await?;

    // still returns error
    let has_followers = CommunityFollowerView::check_has_followers_from_instance(
      community.id,
      remote_instance.id,
      pool,
    )
    .await;
    assert!(has_followers.is_err());

    // mark follower as accepted
    follower_form.follow_state = CommunityFollowerState::Accepted;
    CommunityActions::follow(pool, &follower_form).await?;

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
