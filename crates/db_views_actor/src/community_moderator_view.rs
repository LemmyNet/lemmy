use crate::structs::CommunityModeratorView;
use diesel::{dsl::exists, result::Error, select, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::{community, community_actions, person},
  utils::{get_conn, DbPool},
  CommunityVisibility,
};

impl CommunityModeratorView {
  pub async fn is_community_moderator(
    pool: &mut DbPool<'_>,
    find_community_id: CommunityId,
    find_person_id: PersonId,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      community_actions::table
        .find((find_person_id, find_community_id))
        .filter(community_actions::became_moderator.is_not_null()),
    ))
    .get_result::<bool>(conn)
    .await
  }

  pub(crate) async fn is_community_moderator_of_any(
    pool: &mut DbPool<'_>,
    find_person_id: PersonId,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      community_actions::table
        .filter(community_actions::person_id.eq(find_person_id))
        .filter(community_actions::became_moderator.is_not_null()),
    ))
    .get_result::<bool>(conn)
    .await
  }

  pub async fn for_community(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    community_actions::table
      .inner_join(community::table)
      .inner_join(person::table)
      .filter(community_actions::community_id.eq(community_id))
      .filter(community_actions::became_moderator.is_not_null())
      .select((community::all_columns, person::all_columns))
      .order_by(community_actions::became_moderator)
      .load::<CommunityModeratorView>(conn)
      .await
  }

  pub async fn for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    is_authenticated: bool,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut query = community_actions::table
      .inner_join(community::table)
      .inner_join(person::table)
      .filter(community_actions::person_id.eq(person_id))
      .filter(community_actions::became_moderator.is_not_null())
      .filter(community::deleted.eq(false))
      .filter(community::removed.eq(false))
      .select((community::all_columns, person::all_columns))
      .into_boxed();
    if !is_authenticated {
      query = query.filter(community::visibility.eq(CommunityVisibility::Public));
    }
    query.load::<CommunityModeratorView>(conn).await
  }

  /// Finds all communities first mods / creators
  /// Ideally this should be a group by, but diesel doesn't support it yet
  pub async fn get_community_first_mods(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    community_actions::table
      .inner_join(community::table)
      .inner_join(person::table)
      .filter(community_actions::became_moderator.is_not_null())
      .select((community::all_columns, person::all_columns))
      // A hacky workaround instead of group_bys
      // https://stackoverflow.com/questions/24042359/how-to-join-only-one-row-in-joined-table-with-postgres
      .distinct_on(community_actions::community_id)
      .order_by((
        community_actions::community_id,
        community_actions::person_id,
      ))
      .load::<CommunityModeratorView>(conn)
      .await
  }
}
