use extism::{Manifest, Plugin};
use lemmy_api_common::LemmyErrorType;
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

pub fn plugin_hook<T>(name: &'static str, data: &mut T) -> LemmyResult<()>
where
  T: Clone + Serialize + for<'a> Deserialize<'a>,
{
  // TODO: use std::sync::OnceLock once get_mut_or_init() is stabilized
  // https://doc.rust-lang.org/std/sync/struct.OnceLock.html#method.get_mut_or_init
  let mut plugins = Lazy::new(|| Plugins::load());
  plugins.call(name, data)?;
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
  pub fn load() -> Self {
    let dir = env::var("LEMMY_PLUGIN_PATH").unwrap_or("plugins".to_string());
    let plugin_paths = read_dir(dir).expect("read plugin folder");

    let plugins = plugin_paths
      .flat_map(|p| p.ok())
      .map(|p| p.path())
      .filter(|p| p.extension() == Some(OsStr::new("manifest")))
      .flat_map(|p| {
        init_plugin(&p)
          .inspect_err(|e| warn!("Failed to load plugin {}: {e}", p.to_string_lossy()))
          .ok()
      })
      .collect();
    Self { plugins }
  }

  pub fn call<T>(&mut self, name: &str, data: &mut T) -> LemmyResult<()>
  where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
  {
    debug!("Calling plugin hook {name}");
    for p in &mut self.plugins {
      if p.function_exists(name) {
        *data = p
          .call::<extism_convert::Json<T>, extism_convert::Json<T>>(name, (*data).clone().into())
          .map_err(|e| LemmyErrorType::PluginError(e.to_string()))?
          .0
          .into();
      }
    }
    Ok(())
  }
}
