use crate::structs::CommunityModeratorView;
use diesel::{dsl::exists, result::Error, select, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CommunityId, PersonId},
  schema::{community, community_moderator, person},
  source::local_user::LocalUser,
  utils::{get_conn, DbPool},
};

impl CommunityModeratorView {
  pub async fn is_community_moderator(
    pool: &mut DbPool<'_>,
    find_community_id: CommunityId,
    find_person_id: PersonId,
  ) -> Result<bool, Error> {
    use lemmy_db_schema::schema::community_moderator::dsl::{
      community_id,
      community_moderator,
      person_id,
    };
    let conn = &mut get_conn(pool).await?;
    select(exists(
      community_moderator
        .filter(community_id.eq(find_community_id))
        .filter(person_id.eq(find_person_id)),
    ))
    .get_result::<bool>(conn)
    .await
  }

  pub(crate) async fn is_community_moderator_of_any(
    pool: &mut DbPool<'_>,
    find_person_id: PersonId,
  ) -> Result<bool, Error> {
    use lemmy_db_schema::schema::community_moderator::dsl::{community_moderator, person_id};
    let conn = &mut get_conn(pool).await?;
    select(exists(
      community_moderator.filter(person_id.eq(find_person_id)),
    ))
    .get_result::<bool>(conn)
    .await
  }

  pub async fn for_community(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    community_moderator::table
      .inner_join(community::table)
      .inner_join(person::table)
      .filter(community_moderator::community_id.eq(community_id))
      .select((community::all_columns, person::all_columns))
      .order_by(community_moderator::published)
      .load::<CommunityModeratorView>(conn)
      .await
  }

  pub async fn for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    local_user: Option<&LocalUser>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut query = community_moderator::table
      .inner_join(community::table)
      .inner_join(person::table)
      .filter(community_moderator::person_id.eq(person_id))
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
    community_moderator::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((community::all_columns, person::all_columns))
      // A hacky workaround instead of group_bys
      // https://stackoverflow.com/questions/24042359/how-to-join-only-one-row-in-joined-table-with-postgres
      .distinct_on(community_moderator::community_id)
      .order_by((
        community_moderator::community_id,
        community_moderator::published,
      ))
      .load::<CommunityModeratorView>(conn)
      .await
  }
}
