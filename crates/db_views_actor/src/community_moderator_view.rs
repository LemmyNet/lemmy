use crate::structs::CommunityModeratorView;
use diesel::{result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::{community, community_moderator, person},
  source::{
    community::{Community, CommunitySafe},
    person::{Person, PersonSafe},
  },
  traits::{ToSafe, ViewToVec},
  utils::{get_conn, DbPool},
};

type CommunityModeratorViewTuple = (CommunitySafe, PersonSafe);

impl CommunityModeratorView {
  pub async fn for_community(pool: &DbPool, community_id: CommunityId) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = community_moderator::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((
        Community::safe_columns_tuple(),
        Person::safe_columns_tuple(),
      ))
      .filter(community_moderator::community_id.eq(community_id))
      .order_by(community_moderator::published)
      .load::<CommunityModeratorViewTuple>(conn)
      .await?;

    Ok(Self::from_tuple_to_vec(res))
  }

  pub async fn for_person(pool: &DbPool, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = community_moderator::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((
        Community::safe_columns_tuple(),
        Person::safe_columns_tuple(),
      ))
      .filter(community_moderator::person_id.eq(person_id))
      .filter(community::deleted.eq(false))
      .filter(community::removed.eq(false))
      .order_by(community_moderator::published)
      .load::<CommunityModeratorViewTuple>(conn)
      .await?;

    Ok(Self::from_tuple_to_vec(res))
  }

  /// Finds all communities first mods / creators
  /// Ideally this should be a group by, but diesel doesn't support it yet
  pub async fn get_community_first_mods(pool: &DbPool) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = community_moderator::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((
        Community::safe_columns_tuple(),
        Person::safe_columns_tuple(),
      ))
      // A hacky workaround instead of group_bys
      // https://stackoverflow.com/questions/24042359/how-to-join-only-one-row-in-joined-table-with-postgres
      .distinct_on(community_moderator::community_id)
      .order_by((
        community_moderator::community_id,
        community_moderator::person_id,
      ))
      .load::<CommunityModeratorViewTuple>(conn)
      .await?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for CommunityModeratorView {
  type DbTuple = CommunityModeratorViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        community: a.0,
        moderator: a.1,
      })
      .collect::<Vec<Self>>()
  }
}
