use crate::{MultiCommunityView, MultiCommunityViewApub};
use diesel::{
  dsl::{delete, insert_into, sql},
  result::Error,
  sql_types::{Array, Integer, Text},
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use lemmy_db_schema::{
  newtypes::{DbUrl, MultiCommunityId},
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::{community, multi_community, multi_community_entry, person};
use lemmy_utils::error::LemmyResult;

pub enum ReadParams {
  Name {
    user_name: String,
    multi_name: String,
  },
  Id(MultiCommunityId),
  ApId(DbUrl),
}
impl MultiCommunityView {
  pub async fn read(pool: &mut DbPool<'_>, params: ReadParams) -> LemmyResult<MultiCommunityView> {
    let conn = &mut get_conn(pool).await?;
    let mut query = multi_community_entry::table
      .left_join(multi_community::table.left_join(person::table))
      .filter(multi_community::id.is_not_null())
      .group_by(multi_community::id)
      .select((
        multi_community::all_columns.assume_not_null(),
        sql::<Array<Integer>>("array_agg(multi_community_entry.community_id)"),
      ))
      .into_boxed();

    query = match params {
      ReadParams::Name {
        user_name,
        multi_name,
      } => query
        .filter(person::name.eq(user_name))
        .filter(multi_community::name.eq(multi_name)),
      ReadParams::Id(id) => query.filter(multi_community::id.eq(id)),
      ReadParams::ApId(ap_id) => query.filter(multi_community::ap_id.eq(ap_id)),
    };
    let (multi, entries) = query.first(conn).await?;
    Ok(MultiCommunityView { multi, entries })
  }
}

impl MultiCommunityViewApub {
  pub async fn read_apub(
    pool: &mut DbPool<'_>,
    id: MultiCommunityId,
  ) -> LemmyResult<MultiCommunityViewApub> {
    let conn = &mut get_conn(pool).await?;
    let (multi, entries) = multi_community_entry::table
      .inner_join(community::table)
      .left_join(multi_community::table.left_join(person::table))
      .filter(multi_community::id.is_not_null())
      .group_by(multi_community::id)
      .select((
        multi_community::all_columns.assume_not_null(),
        sql::<Array<Text>>("array_agg(community.ap_id)"),
      ))
      .filter(multi_community::id.eq(id))
      .first(conn)
      .await?;
    Ok(MultiCommunityViewApub { multi, entries })
  }
}
