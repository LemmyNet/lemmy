#[macro_use]
extern crate derive_new;

use crate::{note::MyPost, person::MyUser};
use anyhow::Error;
use lemmy_apub_lib::{object_id::ObjectId, signatures::generate_actor_keypair};
use lemmy_utils::LemmyError;
use url::Url;

mod activities;
mod note;
mod person;

#[tokio::main]
async fn main() -> Result<(), LemmyError> {
  let alpha = MyUser::new(
    ObjectId::new(Url::parse("http://localhost:8001/user/alpha")?),
    generate_actor_keypair()?,
  );
  let beta = MyUser::new(
    ObjectId::new(Url::parse("http://localhost:8001/user/beta")?),
    generate_actor_keypair()?,
  );

  alpha.follow(&beta).await?;

  let post_id = ObjectId::new(Url::parse("http://localhost:8001/user/beta")?);
  let sent_post = MyPost::new("hello world!".to_string(), post_id, beta.ap_id.clone());
  beta.post(sent_post.clone()).await?;
  let received_post = alpha.known_posts.first().unwrap();
  assert_eq!(received_post.text, sent_post.text);
  assert_eq!(received_post.ap_id.inner(), sent_post.ap_id.inner());
  assert_eq!(received_post.creator.inner(), sent_post.creator.inner());
  Ok(())
}
