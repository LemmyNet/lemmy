#![allow(clippy::unwrap_used)]

use crate::{context::LemmyContext, request::build_user_agent};
use lemmy_db_schema::{source::secret::Secret, utils::build_db_pool_for_tests};
use lemmy_utils::{
  email::EmailSender,
  rate_limit::{RateLimitCell, RateLimitConfig},
  settings::SETTINGS,
};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use std::sync::Arc;

pub async fn create_context(email_sender: Arc<dyn EmailSender + Send + Sync>) -> LemmyContext {
  let pool = build_db_pool_for_tests().await;

  let settings = SETTINGS.clone();
  let client = Client::builder()
    .user_agent(build_user_agent(&settings))
    .build()
    .unwrap();

  let client = ClientBuilder::new(client).build();
  let secret = Secret {
    id: 0,
    jwt_secret: String::new(),
  };

  let rate_limit_config = RateLimitConfig::builder().build();
  let rate_limit_cell = RateLimitCell::new(rate_limit_config).await;

  LemmyContext::create_non_default(pool, client, secret, rate_limit_cell.clone(), email_sender)
}
