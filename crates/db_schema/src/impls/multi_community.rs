use crate::{
  newtypes::{CommunityId, MultiCommunityId, PersonId},
  source::multi_community::{MultiCommunity, MultiCommunityInsertForm, MultiCommunityUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{delete, insert_into},
  result::Error,
  update,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::{multi_community, multi_community_entry};
use lemmy_utils::error::LemmyResult;

impl Crud for MultiCommunity {
  type InsertForm = MultiCommunityInsertForm;
  type UpdateForm = MultiCommunityUpdateForm;
  type IdType = MultiCommunityId;

  async fn create(pool: &mut DbPool<'_>, form: &MultiCommunityInsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      insert_into(multi_community::table)
        .values(form)
        .get_result(conn)
        .await?,
    )
  }

  async fn update(
    pool: &mut DbPool<'_>,
    id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      update(multi_community::table.find(id))
        .set(form)
        .get_result(conn)
        .await?,
    )
  }
}

impl MultiCommunity {
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

  /// Should be called in a transaction together with update() or upsert()
  pub async fn update_entries(
    pool: &mut DbPool<'_>,
    id: MultiCommunityId,
    new_communities: &Vec<CommunityId>,
  ) -> LemmyResult<()> {
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
      query = query.filter(multi_community::creator_id.eq(owner_id));
    }
    query.get_results(conn).await
  }
}
