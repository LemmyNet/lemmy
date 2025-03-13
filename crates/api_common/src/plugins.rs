use crate::LemmyErrorType;
use anyhow::anyhow;
use extism::{FromBytes, Manifest, PluginBuilder, Pool};
use extism_convert::Json;
use lemmy_utils::error::LemmyResult;
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
use tracing::{debug, warn};
use ts_rs::TS;
use url::Url;

const GET_PLUGIN_TIMEOUT: Duration = Duration::from_secs(0);

/// Call a plugin hook without rewriting data
pub fn plugin_hook<T>(name: &'static str, data: &T) -> LemmyResult<()>
where
  T: Clone + Serialize + for<'a> Deserialize<'a>,
{
  // TODO: as this doesnt return any data it could run multiple plugins in parallel
  //       and/or in background thread
  Plugins::get().call(name, data)?;
  Ok(())
}

/// Call a plugin hook which can rewrite data
pub fn plugin_hook_mut<T>(name: &'static str, data: &mut T) -> LemmyResult<()>
where
  T: Clone + Serialize + for<'a> Deserialize<'a>,
{
  let res = Plugins::get().call(name, data)?;
  if let Some(res) = res {
    *data = res;
  }
  Ok(())
}

pub fn plugin_metadata() -> Vec<PluginMetadata> {
  Plugins::get().0.into_iter().map(|p| p.metadata).collect()
}

#[derive(Serialize, Deserialize, FromBytes, Debug, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
#[encoding(Json)]
pub struct PluginMetadata {
  name: String,
  url: Url,
  description: String,
}

#[derive(Clone)]
struct Plugins(Vec<LemmyPlugin>);

#[derive(Clone)]
struct LemmyPlugin {
  plugin_pool: Pool<()>,
  metadata: PluginMetadata,
}

impl LemmyPlugin {
  fn init(path: &PathBuf) -> LemmyResult<Self> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let manifest: Manifest = serde_json::from_reader(reader)?;
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

impl Plugins {
  /// Load and initialize all plugins
  fn get() -> Self {
    // TODO: use std::sync::OnceLock once get_mut_or_init() is stabilized
    // https://doc.rust-lang.org/std/sync/struct.OnceLock.html#method.get_mut_or_init
    static PLUGINS: Lazy<Plugins> = Lazy::new(|| {
      let dir = env::var("LEMMY_PLUGIN_PATH").unwrap_or("plugins".to_string());
      let plugin_paths = match read_dir(dir) {
        Ok(r) => r,
        Err(e) => {
          warn!("Failed to read plugin folder: {e}");
          return Plugins(vec![]);
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
      Plugins(plugins)
    });
    PLUGINS.deref().clone()
  }

  /// Call all plugins for a given hook name, taking care not to clone data unnnecessarily.
  fn call<T>(&mut self, name: &str, data: &T) -> LemmyResult<Option<T>>
  where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
  {
    debug!("Calling plugin hook {name}");
    // Check if there is any plugin active for this hook, to avoid unnecessary data cloning
    // TODO: not currently supported by pool
    /*
    if !self.0.iter().any(|p| p.plugin_pool.function_exists(name)) {
      return Ok(None);
    }
    */

    let mut res: Json<T> = data.clone().into();
    for p in &mut self.0 {
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
    Ok(Some(res.0))
  }
}
