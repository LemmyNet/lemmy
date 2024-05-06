use actix_http::h1::Payload;
use actix_web::{
  body::MessageBody,
  dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
  web::Bytes,
  Error,
};
use core::future::Ready;
use extism::{Manifest, Plugin};
use futures_util::future::LocalBoxFuture;
use lemmy_utils::{error::LemmyResult, LemmyErrorType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{ffi::OsStr, fs::read_dir, future::ready, rc::Rc};
use tracing::info;

#[derive(Clone)]
pub struct PluginMiddleware {}

impl PluginMiddleware {
  pub fn new() -> Self {
    PluginMiddleware {}
  }
}
impl<S, B> Transform<S, ServiceRequest> for PluginMiddleware
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
  S::Future: 'static,
  B: MessageBody + 'static,
{
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Transform = SessionService<S>;
  type InitError = ();
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ready(Ok(SessionService {
      service: Rc::new(service),
    }))
  }
}

pub struct SessionService<S> {
  service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SessionService<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

  forward_ready!(service);

  fn call(&self, mut service_req: ServiceRequest) -> Self::Future {
    let svc = self.service.clone();

    Box::pin(async move {
      let method = service_req.method();
      let path = service_req.path().replace("/api/v3/", "").replace("/", "_");
      // TODO: naming can be a bit silly, `POST /api/v3/post` becomes `api_before_post_post`
      let plugin_hook = format!("api_before_{method}_{path}").to_lowercase();
      info!("Calling plugin hook {}", &plugin_hook);
      if let Some(mut plugins) = load_plugins()? {
        if plugins.function_exists(&plugin_hook) {
          let payload = service_req.extract::<Bytes>().await?;

          let mut json: Value = serde_json::from_slice(&payload.to_vec())?;
          call_plugin(plugins, &plugin_hook, &mut json)?;

          let (_, mut new_payload) = Payload::create(true);
          new_payload.unread_data(Bytes::from(serde_json::to_vec_pretty(&json)?));
          service_req.set_payload(new_payload.into());
        }
      }
      let res = svc.call(service_req).await?;
      Ok(res)
    })
  }
}

fn load_plugins() -> LemmyResult<Option<Plugin>> {
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
