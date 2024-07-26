use crate::structs::CommunityModeratorView;
use diesel::{dsl::exists, result::Error, select, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CommunityId, PersonId},
  schema::{community, community_actions, person},
  source::local_user::LocalUser,
  utils::{action_query, find_action, get_conn, DbPool},
};

impl CommunityModeratorView {
  pub async fn is_community_moderator(
    pool: &mut DbPool<'_>,
    find_community_id: CommunityId,
    find_person_id: PersonId,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    select(exists(find_action(
      community_actions::became_moderator,
      (find_person_id, find_community_id),
    )))
    .get_result::<bool>(conn)
    .await
  }

  pub(crate) async fn is_community_moderator_of_any(
    pool: &mut DbPool<'_>,
    find_person_id: PersonId,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      action_query(community_actions::became_moderator)
        .filter(community_actions::person_id.eq(find_person_id)),
    ))
    .get_result::<bool>(conn)
    .await
  }

  pub async fn for_community(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    action_query(community_actions::became_moderator)
      .inner_join(community::table)
      .inner_join(person::table)
      .filter(community_actions::community_id.eq(community_id))
      .select((community::all_columns, person::all_columns))
      .order_by(community_actions::became_moderator)
      .load::<CommunityModeratorView>(conn)
      .await
  }

  pub async fn for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    local_user: Option<&LocalUser>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut query = action_query(community_actions::became_moderator)
      .inner_join(community::table)
      .inner_join(person::table)
      .filter(community_actions::person_id.eq(person_id))
      .select((community::all_columns, person::all_columns))
      .into_boxed();

    query = local_user.visible_communities_only(query);

    // only show deleted communities to creator
    if Some(person_id) != local_user.person_id() {
      query = query.filter(community::deleted.eq(false));
    }

    // Show removed communities to admins only
    if !local_user.is_admin() {
      query = query.filter(community::removed.eq(false))
    }

    query.load::<CommunityModeratorView>(conn).await
  }

  /// Finds all communities first mods / creators
  /// Ideally this should be a group by, but diesel doesn't support it yet
  pub async fn get_community_first_mods(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    action_query(community_actions::became_moderator)
      .inner_join(community::table)
      .inner_join(person::table)
      .select((community::all_columns, person::all_columns))
      // A hacky workaround instead of group_bys
      // https://stackoverflow.com/questions/24042359/how-to-join-only-one-row-in-joined-table-with-postgres
      .distinct_on(community_actions::community_id)
      .order_by((
        community_actions::community_id,
        community_actions::became_moderator,
      ))
      .load::<CommunityModeratorView>(conn)
      .await
  }
}
