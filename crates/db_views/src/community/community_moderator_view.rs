use crate::structs::CommunityModeratorView;
use diesel::{dsl::exists, result::Error, select, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CommunityId, PersonId},
  schema::{community, community_actions, person},
  source::local_user::LocalUser,
  utils::{action_query, get_conn, DbPool},
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[diesel::dsl::auto_type]
fn joins() -> _ {
  community_actions::table
    .filter(community_actions::became_moderator.is_not_null())
    .inner_join(community::table)
    .inner_join(person::table.on(person::id.eq(community_actions::person_id)))
}

#[diesel::dsl::auto_type]
fn find_person(person_id: PersonId) -> _ {
  community_actions::person_id.eq(person_id)
}

#[diesel::dsl::auto_type]
fn find_community(community_id: CommunityId) -> _ {
  community_actions::community_id.eq(community_id)
}

const SELECTION: (
  <community::table as diesel::Table>::AllColumns,
  <person::table as diesel::Table>::AllColumns,
) = (community::all_columns, person::all_columns);

impl CommunityModeratorView {
  pub async fn check_is_community_moderator(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    person_id: PersonId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      joins()
        .filter(find_person(person_id))
        .filter(find_community(community_id)),
    ))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::NotAModerator.into())
  }

  pub(crate) async fn is_community_moderator_of_any(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      action_query(community_actions::became_moderator).filter(find_person(person_id)),
    ))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::NotAModerator.into())
  }

  pub async fn for_community(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    joins()
      .filter(find_community(community_id))
      .select(SELECTION)
      .order_by(community_actions::became_moderator)
      .load::<Self>(conn)
      .await
  }

  pub async fn for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    local_user: Option<&LocalUser>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut query = joins().filter(find_person(person_id)).into_boxed();

    query = local_user.visible_communities_only(query);

    // only show deleted communities to creator
    if Some(person_id) != local_user.person_id() {
      query = query.filter(community::deleted.eq(false));
    }

    // Show removed communities to admins only
    if !local_user.is_admin() {
      query = query.filter(community::removed.eq(false))
    }

    query.select(SELECTION).load::<Self>(conn).await
  }

  /// Finds all communities first mods / creators
  /// Ideally this should be a group by, but diesel doesn't support it yet
  pub async fn get_community_first_mods(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    joins()
      .select(SELECTION)
      // A hacky workaround instead of group_bys
      // https://stackoverflow.com/questions/24042359/how-to-join-only-one-row-in-joined-table-with-postgres
      .distinct_on(community_actions::community_id)
      .order_by((
        community_actions::community_id,
        community_actions::became_moderator,
      ))
      .load::<Self>(conn)
      .await
  }
}