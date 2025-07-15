use crate::{
  diesel::{ExpressionMethods, QueryDsl},
  newtypes::CommunityId,
  source::community_community_follow::CommunityCommunityFollow,
  utils::{get_conn, DbPool},
};
use diesel::{delete, dsl::insert_into};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::community_community_follow;
use lemmy_utils::error::LemmyResult;

impl CommunityCommunityFollow {
  pub async fn follow(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    follower_id: CommunityId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    insert_into(community_community_follow::table)
      .values((
        community_community_follow::community_id.eq(community_id),
        community_community_follow::follower_id.eq(follower_id),
      ))
      .execute(conn)
      .await?;
    Ok(())
  }

  pub async fn unfollow(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    follower_id: CommunityId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    delete(
      community_community_follow::table
        .filter(community_community_follow::community_id.eq(community_id))
        .filter(community_community_follow::follower_id.eq(follower_id)),
    )
    .execute(conn)
    .await?;
    Ok(())
  }
}
