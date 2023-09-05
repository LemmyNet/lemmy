use crate::structs::CommunityFollowerView;
use diesel::{
    dsl::{count_star, not},
    result::Error,
    sql_function, ExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
    newtypes::{CommunityId, DbUrl, PersonId},
    schema::{community, community_follower, person},
    utils::{get_conn, DbPool},
};

sql_function!(fn coalesce(x: diesel::sql_types::Nullable<diesel::sql_types::Text>, y: diesel::sql_types::Text) -> diesel::sql_types::Text);

impl CommunityFollowerView {
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

    pub async fn for_person(
        pool: &mut DbPool<'_>,
        person_id: PersonId,
    ) -> Result<Vec<Self>, Error> {
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
