use crate::{
  newtypes::{CommunityId, MultiCommunityId, PersonId},
  source::multi_community::{MultiCommunity, MultiCommunityInsertForm, MultiCommunityUpdateForm},
  utils::{get_conn, DbConn, DbPool},
};
use diesel::{
  dsl::{delete, insert_into},
  update,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{multi_community, multi_community_entry};
use lemmy_utils::error::LemmyResult;

impl MultiCommunity {
  pub async fn create(pool: &mut DbPool<'_>, form: &MultiCommunityInsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      insert_into(multi_community::table)
        .values(form)
        .get_result(conn)
        .await?,
    )
  }

  /// Cant use `impl Crud` because this must take DbConn instead of DbPool to work in transactions
  pub async fn update(
    conn: &mut DbConn<'_>,
    id: MultiCommunityId,
    form: &MultiCommunityUpdateForm,
  ) -> LemmyResult<Self> {
    Ok(
      update(multi_community::table.find(id))
        .set(form)
        .get_result(conn)
        .await?,
    )
  }
  pub async fn upsert(conn: &mut DbConn<'_>, form: &MultiCommunityInsertForm) -> LemmyResult<Self> {
    Ok(
      insert_into(multi_community::table)
        .values(form)
        .on_conflict(multi_community::ap_id)
        .do_update()
        .set(form)
        .get_result::<Self>(conn)
        .await?,
    )
  }

  /// Should be called in a transaction together with update() or upsert()
  pub async fn update_entries(
    conn: &mut DbConn<'_>,
    id: MultiCommunityId,
    new_communities: &Vec<CommunityId>,
  ) -> LemmyResult<()> {
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
      .await?;
    Ok(())
  }

  pub async fn list(pool: &mut DbPool<'_>, owner_id: Option<PersonId>) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    let mut query = multi_community::table.into_boxed();
    if let Some(owner_id) = owner_id {
      query = query.filter(multi_community::creator_id.eq(owner_id));
    }
    Ok(query.get_results(conn).await?)
  }
}
