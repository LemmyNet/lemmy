use crate::{
  newtypes::PluginId,
  source::plugin::{Plugin, PluginForm},
};
use diesel::{QueryDsl, insert_into};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::plugin;
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  traits::Crud,
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Crud for Plugin {
  type InsertForm = PluginForm;

  type UpdateForm = PluginForm;

  type IdType = PluginId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(plugin::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    plugin_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(plugin::table.find(plugin_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl Plugin {
  pub async fn read_all(pool: &mut DbPool<'_>) -> LemmyResult<Vec<Plugin>> {
    let conn = &mut get_conn(pool).await?;
    Ok(plugin::table.get_results(conn).await?)
  }
}
