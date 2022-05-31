use crate::{
  error::Error,
  objects::{
    note::MyPost,
    person::{MyUser, PersonAcceptedActivities},
  },
  ObjectId,
};
use activitypub_federation::{
  context::WithContext,
  data::Data,
  inbox::receive_activity,
  traits::ApubObject,
  InstanceSettings,
  LocalInstance,
  APUB_JSON_CONTENT_TYPE,
};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use http_signature_normalization_actix::prelude::VerifyDigest;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::{ops::Deref, sync::Arc};
use url::Url;

pub struct Instance {
  local_instance: Arc<LocalInstance>,
  users: Vec<MyUser>,
  posts: Vec<MyPost>,
}

impl Instance {
  pub fn new(hostname: String) -> Instance {
    let local_instance = LocalInstance::new(
      hostname.clone(),
      Client::default().into(),
      InstanceSettings::default(),
    );
    Instance {
      local_instance: Arc::new(local_instance),
      users: vec![],
      posts: vec![],
    }
  }

  pub fn get_user(&self) -> &MyUser {
    self.users.iter().find(|u| u.local).unwrap()
  }

  pub fn get_all_posts(&self) -> &Vec<MyPost> {
    &self.posts
  }

  pub fn get_local_instance(&self) -> &LocalInstance {
    &self.local_instance
  }

  pub async fn listen(&self) -> Result<(), Error> {
    let local_instance = self.local_instance.clone();
    HttpServer::new(move || {
      App::new()
        .app_data(Data::new(local_instance.clone()))
        .route("/objects/{user_name}", web::get().to(get_user))
        .service(
          web::scope("")
            // TODO: this seems entirely unnecessary as http sig is verifyed in receive_activity
            .wrap(VerifyDigest::new(Sha256::new()))
            .route("/u/{user_name}/inbox", web::post().to(post_inbox)),
        )
    })
    .bind(self.local_instance.hostname())?
    .run()
    .await?;
    Ok(())
  }
}

async fn get_user(request: HttpRequest) -> Result<HttpResponse, Error> {
  let url = Url::parse(&request.uri().to_string())?;
  let data = ObjectId::<MyUser>::new(url)
    .dereference_local(&())
    .await?
    .into_apub(&())
    .await?;
  Ok(
    HttpResponse::Ok()
      .content_type(APUB_JSON_CONTENT_TYPE)
      .json(WithContext::new_default(data)),
  )
}

async fn post_inbox(
  request: HttpRequest,
  payload: String,
  local_instance: web::Data<Arc<LocalInstance>>,
) -> Result<HttpResponse, Error> {
  let activity = serde_json::from_str(&payload)?;
  Ok(
    receive_activity::<WithContext<PersonAcceptedActivities>, MyUser, (), Error>(
      request,
      activity,
      local_instance.deref(),
      &Data::new(()),
    )
    .await?,
  )
}
