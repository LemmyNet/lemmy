use crate::protocol::Source;
use html2md::parse_html;

pub mod comment;
pub mod community;
pub mod instance;
pub mod person;
pub mod post;
pub mod private_message;

pub(crate) fn get_summary_from_string_or_source(
  raw: &Option<String>,
  source: &Option<Source>,
) -> Option<String> {
  if let Some(source) = &source {
    Some(source.content.clone())
  } else {
    raw.as_ref().map(|s| parse_html(s))
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use actix::Actor;
  use background_jobs::QueueHandle;
  use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
  };
  use lemmy_db_schema::{
    establish_unpooled_connection,
    get_database_url_from_env,
    source::secret::Secret,
  };
  use lemmy_utils::{
    rate_limit::{rate_limiter::RateLimiter, RateLimit},
    request::build_user_agent,
    settings::structs::Settings,
    LemmyError,
  };
  use lemmy_websocket::{chat_server::ChatServer, LemmyContext};
  use reqwest::Client;
  use reqwest_middleware::ClientBuilder;
  use std::sync::Arc;
  use tokio::sync::Mutex;

  // TODO: would be nice if we didnt have to use a full context for tests.
  //       or at least write a helper function so this code is shared with main.rs
  pub(crate) fn init_context(activity_queue: QueueHandle) -> LemmyContext {
    // call this to run migrations
    establish_unpooled_connection();
    let settings = Settings::init().unwrap();
    let rate_limiter = RateLimit {
      rate_limiter: Arc::new(Mutex::new(RateLimiter::default())),
      rate_limit_config: settings.rate_limit.to_owned().unwrap_or_default(),
    };
    let client = Client::builder()
      .user_agent(build_user_agent(&settings))
      .build()
      .unwrap();

    let client = ClientBuilder::new(client).build();
    let secret = Secret {
      id: 0,
      jwt_secret: "".to_string(),
    };
    let db_url = match get_database_url_from_env() {
      Ok(url) => url,
      Err(_) => settings.get_database_url(),
    };
    let manager = ConnectionManager::<PgConnection>::new(&db_url);
    let pool = Pool::builder()
      .max_size(settings.database.pool_size)
      .build(manager)
      .unwrap_or_else(|_| panic!("Error connecting to {}", db_url));
    async fn x() -> Result<String, LemmyError> {
      Ok("".to_string())
    }
    let chat_server = ChatServer::startup(
      pool.clone(),
      rate_limiter,
      |_, _, _, _| Box::pin(x()),
      |_, _, _, _| Box::pin(x()),
      client.clone(),
      activity_queue.clone(),
      settings.clone(),
      secret.clone(),
    )
    .start();
    LemmyContext::create(pool, chat_server, client, activity_queue, settings, secret)
  }
}
