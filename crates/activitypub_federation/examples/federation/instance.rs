use crate::{
  error::Error,
  generate_object_id,
  objects::{
    note::MyPost,
    person::{MyUser, PersonAcceptedActivities},
  },
};
use activitypub_federation::{
  context::WithContext,
  data::Data,
  inbox::receive_activity,
  object_id::ObjectId,
  signatures::generate_actor_keypair,
  traits::ApubObject,
  InstanceSettingsBuilder,
  LocalInstance,
  APUB_JSON_CONTENT_TYPE,
};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use http_signature_normalization_actix::prelude::VerifyDigest;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::{
  ops::Deref,
  sync::{Arc, Mutex},
};
use tokio::task;
use url::Url;

pub type InstanceHandle = Arc<Instance>;

pub struct Instance {
  /// This holds all library data
  local_instance: LocalInstance,
  /// Our "database" which contains all known users (local and federated)
  pub users: Mutex<Vec<MyUser>>,
  /// Same, but for posts
  pub posts: Mutex<Vec<MyPost>>,
}

impl Instance {
  pub fn new(hostname: String) -> Result<InstanceHandle, Error> {
    let settings = InstanceSettingsBuilder::default()
      .testing_send_sync(true)
      .worker_count(1)
      .build()?;
    let local_instance = LocalInstance::new(hostname.clone(), Client::default().into(), settings);
    let local_user = MyUser::new(generate_object_id(&hostname)?, generate_actor_keypair()?);
    let instance = Arc::new(Instance {
      local_instance,
      users: Mutex::new(vec![local_user]),
      posts: Mutex::new(vec![]),
    });
    Ok(instance)
  }

  pub fn local_user(&self) -> MyUser {
    self.users.lock().unwrap().first().cloned().unwrap()
  }

  pub fn local_instance(&self) -> &LocalInstance {
    &self.local_instance
  }

  pub fn listen(instance: &InstanceHandle) -> Result<(), Error> {
    let hostname = instance.local_instance.hostname();
    let instance = instance.clone();
    let server = HttpServer::new(move || {
      App::new()
        .app_data(web::Data::new(instance.clone()))
        .route("/objects/{user_name}", web::get().to(http_get_user))
        .service(
          web::scope("")
            // Important: this ensures that the activity json matches the hashsum in signed
            // HTTP header
            // TODO: it would be possible to get rid of this by verifying hash in
            //       receive_activity()
            .wrap(VerifyDigest::new(Sha256::new()))
            // Just a single, global inbox for simplicity
            .route("/inbox", web::post().to(http_post_user_inbox)),
        )
    })
    .bind(hostname)?
    .run();
    task::spawn(server);
    Ok(())
  }
}

/// Handles requests to fetch user json over HTTP
async fn http_get_user(
  request: HttpRequest,
  data: web::Data<InstanceHandle>,
) -> Result<HttpResponse, Error> {
  let data: InstanceHandle = data.into_inner().deref().clone();
  let hostname: String = data.local_instance.hostname().to_string();
  let request_url = format!("http://{}{}", hostname, &request.uri().to_string());
  let url = Url::parse(&request_url)?;
  let user = ObjectId::<MyUser>::new(url)
    .dereference_local::<Error>(&data)
    .await?
    .into_apub(&data)
    .await?;
  Ok(
    HttpResponse::Ok()
      .content_type(APUB_JSON_CONTENT_TYPE)
      .json(WithContext::new_default(user)),
  )
}

/// Handles messages received in user inbox
async fn http_post_user_inbox(
  request: HttpRequest,
  payload: String,
  data: web::Data<InstanceHandle>,
) -> Result<HttpResponse, Error> {
  let data: InstanceHandle = data.into_inner().deref().clone();
  let activity = serde_json::from_str(&payload)?;
  receive_activity::<WithContext<PersonAcceptedActivities>, MyUser, InstanceHandle, Error>(
    request,
    activity,
    &data.clone().local_instance,
    &Data::new(data),
  )
  .await
}
