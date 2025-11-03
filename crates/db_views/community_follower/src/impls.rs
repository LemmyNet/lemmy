use crate::{CommunityFollowerView, PendingFollow};
use chrono::Utc;
use diesel::{
  dsl::{count_star, exists, not},
  select,
  ExpressionMethods,
  JoinOnDsl,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, DbUrl, InstanceId, PaginationCursor, PersonId},
  source::community::CommunityActions,
  traits::PaginationCursorBuilder,
  utils::{functions::lower, get_conn, DbPool},
};
use lemmy_db_schema_file::{
  enums::CommunityFollowerState,
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

  pub async fn count_community_followers(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
  ) -> LemmyResult<i32> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(community_actions::community_id.eq(community_id))
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
      // Exclude private community follows which still need to be approved by a mod
      .filter(community_actions::follow_state.ne(CommunityFollowerState::ApprovalRequired))
      .filter(community_actions::follow_state.ne(CommunityFollowerState::Denied))
      .select(Self::as_select())
      .order_by(lower(community::title))
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
    let [(_, person_id), (_, community_id)] = cursor.prefixes_and_ids()?;
    CommunityActions::read(pool, CommunityId(community_id), PersonId(person_id)).await
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
