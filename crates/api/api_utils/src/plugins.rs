use crate::context::LemmyContext;
use anyhow::anyhow;
use extism::{Manifest, PluginBuilder, Pool, PoolPlugin};
use extism_convert::Json;
use lemmy_db_schema::source::{notification::Notification, person::Person};
use lemmy_db_views_notification::NotificationView;
use lemmy_db_views_site::api::PluginMetadata;
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::{
  VERSION,
  error::{LemmyError, LemmyErrorType, LemmyResult},
  settings::SETTINGS,
};
use serde::{Deserialize, Serialize};
use std::{
  env,
  ffi::OsStr,
  fs::{File, read_dir},
  io::BufReader,
  ops::Deref,
  path::PathBuf,
  sync::{LazyLock, OnceLock},
  time::Duration,
};
use tokio::task::spawn_blocking;
use tracing::warn;
use url::Url;

const GET_PLUGIN_TIMEOUT: Duration = Duration::from_secs(1);

/// Call a plugin hook without rewriting data
pub fn plugin_hook_after<T>(name: &'static str, data: &T)
where
  T: Clone + Serialize + for<'b> Deserialize<'b> + Sync + Send + 'static,
{
  let plugins = LemmyPlugins::get_or_init();
  if !plugins.function_exists(name) {
    return;
  }

  let data = data.clone();
  spawn_blocking(move || run_plugin_hook_after(name, data));
}

/// Calls plugin hook for the given notifications Loads additional data via
/// NotificationView, but only if a plugin is active.
pub async fn plugin_hook_notification(
  notifications: Vec<Notification>,
  context: &LemmyContext,
) -> LemmyResult<()> {
  let name = "notification_after_create";
  let plugins = LemmyPlugins::get_or_init();
  if !plugins.function_exists(name) {
    return Ok(());
  }

  for n in notifications {
    let person = Person::read(&mut context.pool(), n.recipient_id).await?;
    let view = NotificationView::read(&mut context.pool(), n.id, &person).await?;
    spawn_blocking(move || run_plugin_hook_after(name, view));
  }
  Ok(())
}

fn run_plugin_hook_after<T>(name: &'static str, data: T) -> LemmyResult<()>
where
  T: Clone + Serialize + for<'b> Deserialize<'b>,
{
  let plugins = LemmyPlugins::get_or_init();
  for p in plugins.0 {
    if let Some(plugin) = p.get(name)? {
      let params: Json<T> = data.clone().into();
      plugin
        .call::<Json<T>, ()>(name, params)
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
  let plugins = LemmyPlugins::get_or_init();
  if !plugins.function_exists(name) {
    return Ok(data);
  }

  spawn_blocking(move || {
    let mut res: Json<T> = data.into();
    for p in plugins.0 {
      if let Some(plugin) = p.get(name)? {
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
  static METADATA: OnceLock<Vec<PluginMetadata>> = OnceLock::new();
  if let Some(m) = METADATA.get() {
    m.clone()
  } else {
    // Loading metadata can take multiple seconds. Do this in background task to avoid blocking
    // /api/v4/site endpoint.
    std::thread::spawn(|| {
      METADATA.get_or_init(|| {
        let mut metadata = vec![];
        for plugin in LemmyPlugins::get_or_init().0 {
          let run = plugin.pool.get(GET_PLUGIN_TIMEOUT).ok().flatten();
          let m = run.and_then(|run| run.call("metadata", 0).ok());
          if let Some(m) = m {
            metadata.push(m);
          } else {
            // Failed to load plugin metadata, use placeholder
            metadata.push(PluginMetadata {
              name: plugin.filename,
              url: Url::parse("http://unknown").expect("valid url"),
              description: String::new(),
            });
          }
        }
        metadata
      });
    });
    // Return empty metadata until loading is finished
    vec![]
  }
}

#[derive(Clone)]
struct LemmyPlugins(Vec<LemmyPlugin>);

#[derive(Clone)]
struct LemmyPlugin {
  pool: Pool,
  filename: String,
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
    let builder = move || PluginBuilder::new(manifest.clone()).with_wasi(true).build();
    let pool = Pool::new(builder);
    let filename = path
      .file_name()
      .expect("has filename")
      .to_string_lossy()
      .to_string();
    Ok(LemmyPlugin { pool, filename })
  }

  fn get(&self, name: &'static str) -> LemmyResult<Option<PoolPlugin>> {
    let p = self
      .pool
      .get(GET_PLUGIN_TIMEOUT)?
      .ok_or(anyhow!("plugin timeout"))?;

    Ok(if p.plugin().function_exists(name) {
      Some(p)
    } else {
      None
    })
  }
}

impl LemmyPlugins {
  /// Load and initialize all plugins
  fn get_or_init() -> Self {
    static PLUGINS: LazyLock<LemmyPlugins> = LazyLock::new(|| {
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
  fn function_exists(&self, name: &'static str) -> bool {
    self.0.iter().any(|p| {
      p.pool
        .function_exists(name, GET_PLUGIN_TIMEOUT)
        .unwrap_or(false)
    })
  }
}
