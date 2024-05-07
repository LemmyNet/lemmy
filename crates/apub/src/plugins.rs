use extism::{Manifest, Plugin};
use lemmy_utils::{error::LemmyResult, LemmyErrorType};
use serde::{Deserialize, Serialize};
use std::{ffi::OsStr, fs::read_dir};

pub fn load_plugins() -> LemmyResult<Option<Plugin>> {
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
  if !wasm_files.is_empty() {
    // TODO: what if theres more than one plugin for the same hook?
    let manifest = Manifest::new(wasm_files);
    let plugin = Plugin::new(manifest, [], true)?;
    Ok(Some(plugin))
  } else {
    Ok(None)
  }
}

pub fn call_plugin<T: Serialize + for<'de> Deserialize<'de> + Clone>(
  mut plugins: Plugin,
  name: &str,
  data: &mut T,
) -> LemmyResult<()> {
  *data = plugins
    .call::<extism_convert::Json<T>, extism_convert::Json<T>>(name, (*data).clone().into())
    .map_err(|e| LemmyErrorType::PluginError(e.to_string()))?
    .0
    .into();
  Ok(())
}
