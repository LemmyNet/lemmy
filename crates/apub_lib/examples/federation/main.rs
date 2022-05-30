use crate::{
  instance::Instance,
  lib::generate_object_id,
  objects::{note::MyPost, person::MyUser},
};
use lemmy_apub_lib::signatures::generate_actor_keypair;
use lemmy_utils::error::LemmyError;
use tokio::task;

mod activities;
mod instance;
mod lib;
mod objects;

#[actix_rt::main]
async fn main() -> Result<(), LemmyError> {
  static ALPHA_HOSTNAME: &str = "localhost:8001";
  static BETA_HOSTNAME: &str = "localhost:8001";
  let alpha = Instance::new(ALPHA_HOSTNAME.to_string());
  let beta = Instance::new(BETA_HOSTNAME.to_string());
  //task::spawn(async move {
  //  alpha.listen().await;
  //});
  let alpha_user = MyUser::new(
    generate_object_id(ALPHA_HOSTNAME)?,
    generate_actor_keypair()?,
  );
  let beta_user = MyUser::new(
    generate_object_id(BETA_HOSTNAME)?,
    generate_actor_keypair()?,
  );

  alpha_user
    .follow(&beta_user, alpha.get_local_instance(), ALPHA_HOSTNAME)
    .await?;
  assert_eq!(
    beta_user.followers(),
    &vec![alpha_user.ap_id.inner().clone()]
  );

  let sent_post = MyPost::new("hello world!".to_string(), beta_user.ap_id.clone());
  beta_user
    .post(sent_post.clone(), beta.get_local_instance(), BETA_HOSTNAME)
    .await?;
  let received_post = alpha.get_all_posts().first().unwrap();
  assert_eq!(received_post.text, sent_post.text);
  assert_eq!(received_post.ap_id.inner(), sent_post.ap_id.inner());
  assert_eq!(received_post.creator.inner(), sent_post.creator.inner());
  Ok(())
}
