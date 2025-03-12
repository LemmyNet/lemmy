use crate::LemmyErrorType;
use extism::{Manifest, Plugin};
use extism_convert::Json;
use lemmy_utils::error::LemmyResult;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
  env,
  ffi::OsStr,
  fs::{read_dir, File},
  io::BufReader,
  path::PathBuf,
};
use tracing::{debug, warn};

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

struct Plugins {
  plugins: Vec<Plugin>,
}

fn init_plugin(path: &PathBuf) -> LemmyResult<Plugin> {
  let file = File::open(path)?;
  let reader = BufReader::new(file);
  let manifest: Manifest = serde_json::from_reader(reader)?;
  Ok(Plugin::new(manifest, [], true)?)
}

impl Plugins {
  /// Load and initialize all plugins
  fn load() -> Lazy<Self> {
    // TODO: use std::sync::OnceLock once get_mut_or_init() is stabilized
    // https://doc.rust-lang.org/std/sync/struct.OnceLock.html#method.get_mut_or_init
    Lazy::new(|| {
      let dir = env::var("LEMMY_PLUGIN_PATH").unwrap_or("plugins".to_string());
      let plugin_paths = match read_dir(dir) {
        Ok(r) => r,
        Err(e) => {
          warn!("Failed to read plugin folder: {e}");
          return Plugins { plugins: vec![] };
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
      Self { plugins }
    })
  }

  /// Call all plugins for a given hook name, taking care not to clone data unnnecessarily.
  fn call<T>(&mut self, name: &str, data: &T) -> LemmyResult<Option<T>>
  where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
  {
    debug!("Calling plugin hook {name}");
    if self.plugins.iter().any(|p| p.function_exists(name)) {
      return Ok(None);
    }

    let mut res: Json<T> = data.clone().into();
    for p in &mut self.plugins {
      if p.function_exists(name) {
        let r = p
          .call(name, res)
          .map_err(|e| LemmyErrorType::PluginError(e.to_string()))?;
        res = r;
      }
    }
    Ok(Some(res.0))
  }
}
