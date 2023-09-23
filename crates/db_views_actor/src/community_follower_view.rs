use crate::structs::CommunityFollowerView;
use chrono::Utc;
use diesel::{
  dsl::{count_star, not},
  result::Error,
  ExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, DbUrl, InstanceId, PersonId},
  schema::{community, community_follower, person},
  utils::{functions::coalesce, get_conn, DbPool},
};

impl CommunityFollowerView {
  /// return a list of local community ids and remote inboxes that at least one user of the given instance has followed
  pub async fn get_instance_followed_community_inboxes(
    pool: &mut DbPool<'_>,
    instance_id: InstanceId,
    published_since: chrono::DateTime<Utc>,
  ) -> Result<Vec<(CommunityId, DbUrl)>, Error> {
    let conn = &mut get_conn(pool).await?;
    // In most cases this will fetch the same url many times (the shared inbox url)
    // PG will only send a single copy to rust, but it has to scan through all follower rows (same as it was before).
    // So on the PG side it would be possible to optimize this further by adding e.g. a new table community_followed_instances (community_id, instance_id)
    // that would work for all instances that support fully shared inboxes.
    // It would be a bit more complicated though to keep it in sync.

    community_follower::table
      .inner_join(community::table)
      .inner_join(person::table)
      .filter(person::instance_id.eq(instance_id))
      .filter(community::local) // this should be a no-op since community_followers table only has local-person+remote-community or remote-person+local-community
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
    community_follower::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((community::all_columns, person::all_columns))
      .filter(community_follower::person_id.eq(person_id))
      .filter(community::deleted.eq(false))
      .filter(community::removed.eq(false))
      .order_by(community::title)
      .load::<CommunityFollowerView>(conn)
      .await
  }
}
