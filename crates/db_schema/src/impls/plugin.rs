use crate::{
  diesel::ExpressionMethods,
  newtypes::PluginId,
  source::plugin::{Plugin, PluginConfig, PluginForm},
};
use diesel::{QueryDsl, insert_into};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{plugin, plugin_config};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  traits::Crud,
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use std::collections::BTreeMap;

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

pub struct PluginView {
  pub plugin: Plugin,
  pub config: BTreeMap<String, String>,
}

impl Plugin {
  pub async fn read_all(pool: &mut DbPool<'_>) -> LemmyResult<Vec<PluginView>> {
    let conn = &mut get_conn(pool).await?;
    let plugins: Vec<Plugin> = plugin::table.get_results(conn).await?;
    let mut res = vec![];
    for plugin in plugins {
      // TODO: should use only a single sql query
      let config: Vec<PluginConfig> = plugin_config::table
        .filter(plugin_config::plugin_id.eq(plugin.id))
        .get_results(conn)
        .await?;
      let config = config.into_iter().map(|c| (c.key, c.value)).collect();
      res.push(PluginView { plugin, config });
    }
    Ok(res)
  }
}
