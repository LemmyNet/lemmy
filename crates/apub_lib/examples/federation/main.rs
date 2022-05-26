#[macro_use]
extern crate derive_new;

use crate::{lib::generate_object_id, note::MyPost, person::MyUser};
use lemmy_apub_lib::{
  object_id::ObjectId,
  signatures::generate_actor_keypair,
  InstanceSettings,
  LocalInstance,
};
use lemmy_utils::LemmyError;
use reqwest::Client;
use reqwest_middleware::ClientWithMiddleware;
use url::Url;

mod activities;
mod lib;
mod note;
mod person;

#[actix_rt::main]
async fn main() -> Result<(), LemmyError> {
  static ALPHA_HOSTNAME: &'static str = "localhost:8001";
  static BETA_HOSTNAME: &'static str = "localhost:8001";
  let client: ClientWithMiddleware = Client::default().into();
  let alpha = LocalInstance::new(
    ALPHA_HOSTNAME.to_string(),
    client.clone(),
    InstanceSettings::default(),
  );
  let beta = LocalInstance::new(
    BETA_HOSTNAME.to_string(),
    client,
    InstanceSettings::default(),
  );
  let alpha_user = MyUser::new(
    ObjectId::new(generate_object_id(ALPHA_HOSTNAME)?),
    generate_actor_keypair()?,
  );
  let beta_user = MyUser::new(
    ObjectId::new(generate_object_id(BETA_HOSTNAME)?),
    generate_actor_keypair()?,
  );

  alpha_user
    .follow(&beta_user, &alpha, ALPHA_HOSTNAME)
    .await?;
  assert_eq!(
    beta_user.followers(),
    &vec![alpha_user.ap_id.inner().clone()]
  );

  let post_id = ObjectId::new(generate_object_id(BETA_HOSTNAME)?);
  let sent_post = MyPost::new("hello world!".to_string(), post_id, beta_user.ap_id.clone());
  beta_user
    .post(sent_post.clone(), &beta, BETA_HOSTNAME)
    .await?;
  let received_post = alpha_user.known_posts.first().unwrap();
  assert_eq!(received_post.text, sent_post.text);
  assert_eq!(received_post.ap_id.inner(), sent_post.ap_id.inner());
  assert_eq!(received_post.creator.inner(), sent_post.creator.inner());
  Ok(())
}
