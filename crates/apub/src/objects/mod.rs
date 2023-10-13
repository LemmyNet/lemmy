use crate::protocol::Source;
use activitypub_federation::protocol::values::MediaTypeMarkdownOrHtml;
use anyhow::anyhow;
use html2md::parse_html;
use lemmy_utils::{error::LemmyError, settings::structs::Settings};
use url::Url;

pub mod comment;
pub mod community;
pub mod instance;
pub mod person;
pub mod post;
pub mod private_message;

pub(crate) fn read_from_string_or_source(
  content: &str,
  media_type: &Option<MediaTypeMarkdownOrHtml>,
  source: &Option<Source>,
) -> String {
  if let Some(s) = source {
    // markdown sent by lemmy in source field
    s.content.clone()
  } else if media_type == &Some(MediaTypeMarkdownOrHtml::Markdown) {
    // markdown sent by peertube in content field
    content.to_string()
  } else {
    // otherwise, convert content html to markdown
    parse_html(content)
  }
}

pub(crate) fn read_from_string_or_source_opt(
  content: &Option<String>,
  media_type: &Option<MediaTypeMarkdownOrHtml>,
  source: &Option<Source>,
) -> Option<String> {
  content
    .as_ref()
    .map(|content| read_from_string_or_source(content, media_type, source))
}

/// When for example a Post is made in a remote community, the community will send it back,
/// wrapped in Announce. If we simply receive this like any other federated object, overwrite the
/// existing, local Post. In particular, it will set the field local = false, so that the object
/// can't be fetched from the Activitypub HTTP endpoint anymore (which only serves local objects).
pub(crate) fn verify_is_remote_object(id: &Url, settings: &Settings) -> Result<(), LemmyError> {
  let local_domain = settings.get_hostname_without_port()?;
  if id.domain() == Some(&local_domain) {
    Err(anyhow!("cant accept local object from remote instance").into())
  } else {
    Ok(())
  }
}

#[cfg(test)]
pub(crate) mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use activitypub_federation::config::{Data, FederationConfig};
  use anyhow::anyhow;
  use lemmy_api_common::{context::LemmyContext, request::build_user_agent};
  use lemmy_db_schema::{source::secret::Secret, utils::build_db_pool_for_tests};
  use lemmy_utils::{rate_limit::RateLimitCell, settings::SETTINGS};
  use reqwest::{Client, Request, Response};
  use reqwest_middleware::{ClientBuilder, Middleware, Next};
  use task_local_extensions::Extensions;

  struct BlockedMiddleware;

  /// A reqwest middleware which blocks all requests
  #[async_trait::async_trait]
  impl Middleware for BlockedMiddleware {
    async fn handle(
      &self,
      _req: Request,
      _extensions: &mut Extensions,
      _next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
      Err(anyhow!("Network requests not allowed").into())
    }
  }

  // TODO: would be nice if we didnt have to use a full context for tests.
  pub(crate) async fn init_context() -> Data<LemmyContext> {
    // call this to run migrations
    let pool = build_db_pool_for_tests().await;

    let settings = SETTINGS.clone();
    let client = Client::builder()
      .user_agent(build_user_agent(&settings))
      .build()
      .unwrap();

    let client = ClientBuilder::new(client).with(BlockedMiddleware).build();
    let secret = Secret {
      id: 0,
      jwt_secret: String::new(),
    };

    let rate_limit_cell = RateLimitCell::default();

    let context = LemmyContext::create(pool, client, secret, rate_limit_cell.clone());
    let config = FederationConfig::builder()
      .domain("example.com")
      .app_data(context)
      .build()
      .await
      .unwrap();
    config.to_request_data()
  }
}
