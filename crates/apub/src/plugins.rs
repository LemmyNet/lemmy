use extism::{Manifest, Plugin};
use lemmy_utils::{error::LemmyResult, LemmyErrorType};
use serde::{Deserialize, Serialize};
use std::{ffi::OsStr, fs::read_dir};
use tracing::info;

pub struct Plugins {
  plugins: Vec<Plugin>,
}

impl Plugins {
  pub fn load() -> LemmyResult<Self> {
    // TODO: make dir configurable via env var
    // TODO: should only read fs once at startup for performance
    let plugin_paths = read_dir("plugins")?;

    let mut wasm_files = vec![];
    for path in plugin_paths {
      let path = path?.path();
      if path.extension() == Some(OsStr::new("wasm")) {
        wasm_files.push(path);
      }
    }
    let plugins = wasm_files
      .into_iter()
      .map(|w| {
        let manifest = Manifest::new(vec![w]);
        Plugin::new(manifest, [], true).unwrap()
      })
      .collect();
    Ok(Self { plugins })
  }

  pub fn exists(&mut self, name: &str) -> bool {
    for p in &mut self.plugins {
      if p.function_exists(name) {
        return true;
      }
    }
    false
  }

  pub fn call<T: Serialize + for<'de> Deserialize<'de> + Clone>(
    &mut self,
    name: &str,
    data: &mut T,
  ) -> LemmyResult<()> {
    info!("Calling plugin hook {name}");
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
