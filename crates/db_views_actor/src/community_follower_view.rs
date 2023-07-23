use crate::structs::CommunityFollowerView;
use chrono::Utc;
use diesel::{
  dsl::{count_star, not},
  result::Error,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, DbUrl, InstanceId, PersonId},
  schema::{community, community_follower, person},
  source::{community::Community, person::Person},
  traits::JoinView,
  utils::{functions::coalesce, get_conn, DbPool},
};

type CommunityFollowerViewTuple = (Community, Person);

impl CommunityFollowerView {
  /// return a list of community ids and inboxes that at least one user of the given instance has followed
  pub async fn get_instance_followed_community_inboxes(
    pool: &mut DbPool<'_>,
    instance_id: InstanceId,
    published_since: chrono::DateTime<Utc>,
  ) -> Result<Vec<(CommunityId, DbUrl)>, Error> {
    let conn = &mut get_conn(pool).await?;
    // todo: in most cases this will fetch the same url many times (the shared inbox url)
    community_follower::table
      .inner_join(community::table)
      .inner_join(person::table)
      .filter(person::instance_id.eq(instance_id))
      .filter(not(person::local))
      .filter(community_follower::published.gt(published_since.naive_utc()))
      .select((
        community::id,
        coalesce(person::shared_inbox_url, person::inbox_url),
      ))
      .distinct() // only need each community_id, inbox combination once
      .load::<(CommunityId, DbUrl)>(conn)
      .await
  }
  pub async fn get_community_follower_inboxes(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
  ) -> Result<Vec<DbUrl>, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = community_follower::table
      .filter(community_follower::community_id.eq(community_id))
      .filter(not(person::local))
      .inner_join(person::table)
      .select(coalesce(person::shared_inbox_url, person::inbox_url))
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
    let res = community_follower::table
      .filter(community_follower::community_id.eq(community_id))
      .select(count_star())
      .first::<i64>(conn)
      .await?;

    Ok(res)
  }

  pub async fn for_person(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = community_follower::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((community::all_columns, person::all_columns))
      .filter(community_follower::person_id.eq(person_id))
      .filter(community::deleted.eq(false))
      .filter(community::removed.eq(false))
      .order_by(community::title)
      .load::<CommunityFollowerViewTuple>(conn)
      .await?;

    Ok(res.into_iter().map(Self::from_tuple).collect())
  }
}

impl JoinView for CommunityFollowerView {
  type JoinTuple = CommunityFollowerViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      community: a.0,
      follower: a.1,
    }
  }
}
