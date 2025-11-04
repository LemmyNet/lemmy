use crate::{
  diesel::{ExpressionMethods, QueryDsl},
  newtypes::CommunityId,
  source::community_community_follow::CommunityCommunityFollow,
};
use diesel::{delete, dsl::insert_into};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::community_community_follow;
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::LemmyResult;

impl CommunityCommunityFollow {
  pub async fn follow(
    pool: &mut DbPool<'_>,
    target_id: CommunityId,
    community_id: CommunityId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    insert_into(community_community_follow::table)
      .values((
        community_community_follow::target_id.eq(target_id),
        community_community_follow::community_id.eq(community_id),
      ))
      .execute(conn)
      .await?;
    Ok(())
  }

  pub async fn unfollow(
    pool: &mut DbPool<'_>,
    target_id: CommunityId,
    community_id: CommunityId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    delete(
      community_community_follow::table
        .filter(community_community_follow::target_id.eq(target_id))
        .filter(community_community_follow::community_id.eq(community_id)),
    )
    .execute(conn)
    .await?;
    Ok(())
  }
}
