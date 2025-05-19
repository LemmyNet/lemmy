use crate::{
  newtypes::{CommunityId, MultiCommunityId, PersonId},
  source::multi_community::{MultiCommunity, MultiCommunityInsertForm},
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{delete, insert_into},
  result::Error,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::{multi_community, multi_community_entry};

impl MultiCommunity {
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

  pub async fn upsert(
    pool: &mut DbPool<'_>,
    form: &MultiCommunityInsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(multi_community::table)
      .values(form)
      .on_conflict(multi_community::ap_id)
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn update(
    pool: &mut DbPool<'_>,
    id: MultiCommunityId,
    new_communities: &Vec<CommunityId>,
  ) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    conn
      .transaction::<_, Error, _>(|conn| {
        async move {
          delete(multi_community_entry::table)
            .filter(multi_community_entry::multi_community_id.eq(id))
            .filter(multi_community_entry::community_id.ne_all(new_communities))
            .execute(conn)
            .await?;
          let forms = new_communities
            .iter()
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

  pub async fn list(pool: &mut DbPool<'_>, owner_id: Option<PersonId>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut query = multi_community::table.into_boxed();
    if let Some(owner_id) = owner_id {
      query = query.filter(multi_community::owner_id.eq(owner_id));
    }
    query.get_results(conn).await
  }
}
