use crate::{
  newtypes::MultiCommunityId,
  source::multi_community::{MultiCommunity, MultiCommunityEntryForm, MultiCommunityInsertForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{delete, insert_into},
  result::Error,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{multi_community, multi_community_entry};
use lemmy_utils::error::LemmyResult;

impl Crud for MultiCommunity {
  type InsertForm = MultiCommunityInsertForm;
  type UpdateForm = ();
  type IdType = MultiCommunityId;

  async fn read(pool: &mut DbPool<'_>, id: MultiCommunityId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    multi_community::table.find(id).first(conn).await
  }

  async fn create(pool: &mut DbPool<'_>, form: &MultiCommunityInsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(multi_community::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  async fn update(
    _pool: &mut DbPool<'_>,
    _id: MultiCommunityId,
    _form: &(),
  ) -> Result<Self, Error> {
    // TODO: mark as deleted/removed?
    unimplemented!()
  }
}

impl MultiCommunity {
  pub async fn add_community(
    pool: &mut DbPool<'_>,
    form: MultiCommunityEntryForm,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    insert_into(multi_community_entry::table)
      .values(form)
      .on_conflict_do_nothing()
      .execute(conn)
      .await?;
    Ok(())
  }

  pub async fn remove_community(
    pool: &mut DbPool<'_>,
    form: MultiCommunityEntryForm,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    delete(
      multi_community_entry::table
        .filter(multi_community_entry::multi_community_id.eq(form.multi_community_id))
        .filter(multi_community_entry::community_id.eq(form.community_id)),
    )
    .execute(conn)
    .await?;
    Ok(())
  }
}
