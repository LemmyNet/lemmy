use crate::{site::PluginMetadata, LemmyErrorType};
use anyhow::anyhow;
use extism::{Manifest, PluginBuilder, Pool};
use extism_convert::Json;
use lemmy_utils::{
  error::{LemmyError, LemmyResult},
  settings::SETTINGS,
  VERSION,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
  env,
  ffi::OsStr,
  fs::{read_dir, File},
  io::BufReader,
  ops::Deref,
  path::PathBuf,
  thread::available_parallelism,
  time::Duration,
};
use tokio::task::spawn_blocking;
use tracing::warn;

const GET_PLUGIN_TIMEOUT: Duration = Duration::from_secs(1);

/// Call a plugin hook without rewriting data
pub fn plugin_hook_after<T>(name: &'static str, data: &T) -> LemmyResult<()>
where
  T: Clone + Serialize + for<'b> Deserialize<'b> + Sync + Send + 'static,
{
  let plugins = LemmyPlugins::init();
  if !plugins.loaded(name) {
    return Ok(());
  }

  let data = data.clone();
  spawn_blocking(move || {
    run_plugin_hook_after(plugins, name, data).inspect_err(|e| warn!("Plugin error: {e}"))
  });
  Ok(())
}

fn run_plugin_hook_after<T>(plugins: LemmyPlugins, name: &'static str, data: T) -> LemmyResult<()>
where
  T: Clone + Serialize + for<'b> Deserialize<'b>,
{
  for p in plugins.0 {
    // TODO: add helper method (requires PoolPlugin to be public)
    // https://github.com/extism/extism/pull/696/files#r2003467812
    let p = p
      .plugin_pool
      .get(&(), GET_PLUGIN_TIMEOUT)?
      .ok_or(anyhow!("plugin timeout"))?;
    if p.plugin().function_exists(name) {
      let params: Json<T> = data.clone().into();
      p.call::<Json<T>, ()>(name, params)
        .map_err(|e| LemmyErrorType::PluginError(e.to_string()))?;
    }
  }
  Ok(())
}

/// Call a plugin hook which can rewrite data
pub async fn plugin_hook_before<T>(name: &'static str, data: T) -> LemmyResult<T>
where
  T: Clone + Serialize + for<'a> Deserialize<'a> + Sync + Send + 'static,
{
  let plugins = LemmyPlugins::init();
  if !plugins.loaded(name) {
    return Ok(data);
  }

  spawn_blocking(move || {
    let mut res: Json<T> = data.into();
    for p in plugins.0 {
      // TODO: add helper method (see above)
      let plugin = p
        .plugin_pool
        .get(&(), GET_PLUGIN_TIMEOUT)?
        .ok_or(anyhow!("plugin timeout"))?;
      if plugin.plugin().function_exists(name) {
        let r = plugin
          .call(name, res)
          .map_err(|e| LemmyErrorType::PluginError(e.to_string()))?;
        res = r;
      }
    }
    Ok::<_, LemmyError>(res.0)
  })
  .await?
}

pub fn plugin_metadata() -> Vec<PluginMetadata> {
  LemmyPlugins::init()
    .0
    .into_iter()
    .map(|p| p.metadata)
    .collect()
}

#[derive(Clone)]
struct LemmyPlugins(Vec<LemmyPlugin>);

#[derive(Clone)]
struct LemmyPlugin {
  plugin_pool: Pool<()>,
  metadata: PluginMetadata,
}

impl LemmyPlugin {
  fn init(path: &PathBuf) -> LemmyResult<Self> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut manifest: Manifest = serde_json::from_reader(reader)?;
    manifest.config.insert(
      "lemmy_url".to_string(),
      format!("http://{}:{}/", SETTINGS.bind, SETTINGS.port),
    );
    manifest
      .config
      .insert("lemmy_version".to_string(), VERSION.to_string());
    let plugin_pool: Pool<()> = Pool::new(available_parallelism()?.into());
    let builder = PluginBuilder::new(manifest).with_wasi(true);
    let metadata: PluginMetadata = builder.clone().build()?.call("metadata", 0)?;
    plugin_pool.add_builder((), builder);
    Ok(LemmyPlugin {
      plugin_pool,
      metadata,
    })
  }
}

impl LemmyPlugins {
  /// Load and initialize all plugins
  fn init() -> Self {
    // TODO: use std::sync::OnceLock once get_mut_or_init() is stabilized
    // https://doc.rust-lang.org/std/sync/struct.OnceLock.html#method.get_mut_or_init
    static PLUGINS: Lazy<LemmyPlugins> = Lazy::new(|| {
      let dir = env::var("LEMMY_PLUGIN_PATH").unwrap_or("plugins".to_string());
      let plugin_paths = match read_dir(dir) {
        Ok(r) => r,
        Err(e) => {
          warn!("Failed to read plugin folder: {e}");
          return LemmyPlugins(vec![]);
        }
      };

      let plugins = plugin_paths
        .flat_map(Result::ok)
        .map(|p| p.path())
        .filter(|p| p.extension() == Some(OsStr::new("json")))
        .flat_map(|p| {
          LemmyPlugin::init(&p)
            .inspect_err(|e| warn!("Failed to load plugin {}: {e}", p.to_string_lossy()))
            .ok()
        })
        .collect();
      LemmyPlugins(plugins)
    });
    PLUGINS.deref().clone()
  }

  /// Return early if no plugin is loaded for the given hook name
  fn loaded(&self, _name: &'static str) -> bool {
    // Check if there is any plugin active for this hook, to avoid unnecessary data cloning
    // TODO: not currently supported by pool
    /*
    if !self.0.iter().any(|p| p.plugin_pool.function_exists(name)) {
      return Ok(None);
    }
    */
    !self.0.is_empty()
  }
}
