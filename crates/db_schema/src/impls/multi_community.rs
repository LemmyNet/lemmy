use crate::{
  diesel::NullableExpressionMethods,
  newtypes::{CommunityId, MultiCommunityId, PersonId},
  source::multi_community::{MultiCommunity, MultiCommunityInsertForm, MultiCommunityView},
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{delete, insert_into},
  result::Error,
  sql_types::Array,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::{multi_community, multi_community_entry};

impl MultiCommunity {
  pub async fn read(
    pool: &mut DbPool<'_>,
    id: MultiCommunityId,
  ) -> Result<MultiCommunityView, Error> {
    let conn = &mut get_conn(pool).await?;
    // select multi_community, array_agg(multi_community_entry.community_id) from
    // multi_community_entry left join multi_community on multi_community.id =
    // multi_community_entry.multi_community_id group by multi_community.id;
    let (multi, entries) = multi_community_entry::table
      .left_join(multi_community::table)
      .filter(multi_community::id.is_not_null())
      .filter(multi_community_entry::multi_community_id.eq(id))
      .group_by(multi_community::id)
      .select((
        multi_community::all_columns.assume_not_null(),
        diesel::dsl::sql::<Array<diesel::sql_types::Integer>>(
          "array_agg(multi_community_entry.community_id) community_ids",
        ),
      ))
      .first(conn)
      .await?;
    Ok(MultiCommunityView { multi, entries })
  }

  pub async fn list(pool: &mut DbPool<'_>, owner_id: Option<PersonId>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut query = multi_community::table.into_boxed();
    if let Some(owner_id) = owner_id {
      query = query.filter(multi_community::owner_id.eq(owner_id));
    }
    query.get_results(conn).await
  }

  pub async fn create(
    pool: &mut DbPool<'_>,
    form: &MultiCommunityInsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(multi_community::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn update(
    pool: &mut DbPool<'_>,
    id: MultiCommunityId,
    new_communities: Vec<CommunityId>,
  ) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    conn
      .transaction::<_, Error, _>(|conn| {
        async move {
          delete(multi_community_entry::table)
            .filter(multi_community_entry::multi_community_id.eq(id))
            .filter(multi_community_entry::community_id.ne_all(&new_communities))
            .execute(conn)
            .await?;
          let forms = new_communities
            .into_iter()
            .map(|k| {
              (
                multi_community_entry::multi_community_id.eq(id),
                multi_community_entry::community_id.eq(k),
              )
            })
            .collect::<Vec<_>>();
          insert_into(multi_community_entry::table)
            .values(forms)
            .on_conflict_do_nothing()
            .execute(conn)
            .await
        }
        .scope_boxed()
      })
      .await?;
    Ok(())
  }
}
