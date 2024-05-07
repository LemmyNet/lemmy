use actix_http::{body::BoxBody, h1::Payload};
use actix_web::{
  body::MessageBody,
  dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
  web::Bytes,
  Error,
};
use core::future::Ready;
use futures_util::future::LocalBoxFuture;
use lemmy_apub::plugins::{call_plugin, load_plugins};
use serde_json::Value;
use std::{future::ready, rc::Rc};
use tracing::info;

#[derive(Clone)]
pub struct PluginMiddleware {}

impl PluginMiddleware {
  pub fn new() -> Self {
    PluginMiddleware {}
  }
}
impl<S> Transform<S, ServiceRequest> for PluginMiddleware
where
  S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
  S::Future: 'static,
{
  type Response = ServiceResponse<BoxBody>;
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

impl<S> Service<ServiceRequest> for SessionService<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
  S::Future: 'static,
{
  type Response = ServiceResponse<BoxBody>;
  type Error = Error;
  type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

  forward_ready!(service);

  fn call(&self, mut service_req: ServiceRequest) -> Self::Future {
    let svc = self.service.clone();

    Box::pin(async move {
      let method = service_req.method().clone();
      let path = service_req.path().replace("/api/v3/", "").replace("/", "_");
      // TODO: naming can be a bit silly, `POST /api/v3/post` becomes `api_before_post_post`
      let before_plugin_hook = format!("api_before_{method}_{path}").to_lowercase();

      info!("Calling plugin hook {}", &before_plugin_hook);
      if let Some(mut plugins) = load_plugins()? {
        if plugins.function_exists(&before_plugin_hook) {
          let payload = service_req.extract::<Bytes>().await?;

          let mut json: Value = serde_json::from_slice(&payload.to_vec())?;
          call_plugin(plugins, &before_plugin_hook, &mut json)?;

          let (_, mut new_payload) = Payload::create(true);
          new_payload.unread_data(Bytes::from(serde_json::to_vec(&json)?));
          service_req.set_payload(new_payload.into());
        }
      }
      let mut res = svc.call(service_req).await?;

      // TODO: add after hook
      let after_plugin_hook = format!("api_after_{method}_{path}").to_lowercase();
      info!("Calling plugin hook {}", &after_plugin_hook);
      if let Some(mut plugins) = load_plugins()? {
        if plugins.function_exists(&before_plugin_hook) {
          res = res.map_body(|_, body| {
            let mut json: Value =
              serde_json::from_slice(&body.try_into_bytes().unwrap().to_vec()).unwrap();
            call_plugin(plugins, &after_plugin_hook, &mut json).unwrap();
            BoxBody::new(Bytes::from(serde_json::to_vec(&json).unwrap()))
          });
        }
      }

      Ok(res)
    })
  }
}
