use crate::LemmyErrorType;
use extism::{FromBytes, Manifest, Plugin};
use extism_convert::Json;
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use std::{
  env,
  ffi::OsStr,
  fs::{read_dir, File},
  io::BufReader,
  path::PathBuf,
};
use tracing::{debug, warn};
use ts_rs::TS;
use url::Url;

/// Call a plugin hook without rewriting data
pub fn plugin_hook<T>(name: &'static str, data: &T) -> LemmyResult<()>
where
  T: Clone + Serialize + for<'a> Deserialize<'a>,
{
  Plugins::load().call(name, data)?;
  Ok(())
}

/// Call a plugin hook which can rewrite data
pub fn plugin_hook_mut<T>(name: &'static str, data: &mut T) -> LemmyResult<()>
where
  T: Clone + Serialize + for<'a> Deserialize<'a>,
{
  let res = Plugins::load().call(name, data)?;
  if let Some(res) = res {
    *data = res;
  }
  Ok(())
}

pub fn plugin_metadata() -> Vec<PluginMetadata> {
  Plugins::load().0.into_iter().map(|p| p.metadata).collect()
}

struct Plugins(Vec<LemmyPlugin>);

struct LemmyPlugin {
  plugin: Plugin,
  metadata: PluginMetadata,
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

fn init_plugin(path: &PathBuf) -> LemmyResult<LemmyPlugin> {
  let file = File::open(path)?;
  let reader = BufReader::new(file);
  let manifest: Manifest = serde_json::from_reader(reader)?;
  let mut plugin = Plugin::new(manifest, [], true)?;
  let metadata = plugin.call("metadata", 0)?;
  Ok(LemmyPlugin { plugin, metadata })
}

impl Plugins {
  /// Load and initialize all plugins
  fn load() -> Self {
    // TODO: use std::sync::OnceLock once get_mut_or_init() is stabilized
    // https://doc.rust-lang.org/std/sync/struct.OnceLock.html#method.get_mut_or_init
    //static PLUGINS: Lazy<Plugins> = Lazy::new(|| {
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
        init_plugin(&p)
          .inspect_err(|e| warn!("Failed to load plugin {}: {e}", p.to_string_lossy()))
          .ok()
      })
      .collect();
    Plugins(plugins)
    //});
    // TODO: avoid cloning
    //PLUGINS.deref().clone()
  }

  /// Call all plugins for a given hook name, taking care not to clone data unnnecessarily.
  fn call<T>(&mut self, name: &str, data: &T) -> LemmyResult<Option<T>>
  where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
  {
    debug!("Calling plugin hook {name}");
    if !self.0.iter().any(|p| p.plugin.function_exists(name)) {
      return Ok(None);
    }

    let mut res: Json<T> = data.clone().into();
    for p in &mut self.0 {
      if p.plugin.function_exists(name) {
        let r = p
          .plugin
          .call(name, res)
          .map_err(|e| LemmyErrorType::PluginError(e.to_string()))?;
        res = r;
      }
    }
    Ok(Some(res.0))
  }
}
