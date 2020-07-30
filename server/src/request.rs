use crate::LemmyError;
use anyhow::anyhow;
use std::future::Future;
use thiserror::Error;

#[derive(Clone, Debug, Error)]
#[error("Error sending request, {0}")]
struct SendError(pub String);

#[derive(Clone, Debug, Error)]
#[error("Error receiving response, {0}")]
pub struct RecvError(pub String);

pub async fn retry<F, Fut, T>(f: F) -> Result<T, LemmyError>
where
  F: Fn() -> Fut,
  Fut: Future<Output = Result<T, actix_web::client::SendRequestError>>,
{
  retry_custom(|| async { Ok((f)().await) }).await
}

pub async fn retry_custom<F, Fut, T>(f: F) -> Result<T, LemmyError>
where
  F: Fn() -> Fut,
  Fut: Future<Output = Result<Result<T, actix_web::client::SendRequestError>, LemmyError>>,
{
  let mut response = Err(anyhow!("connect timeout").into());

  for _ in 0u8..3 {
    match (f)().await? {
      Ok(t) => return Ok(t),
      Err(e) => {
        if is_connect_timeout(&e) {
          response = Err(SendError(e.to_string()).into());
          continue;
        }
        return Err(SendError(e.to_string()).into());
      }
    }
  }

  response
}

fn is_connect_timeout(e: &actix_web::client::SendRequestError) -> bool {
  if let actix_web::client::SendRequestError::Connect(e) = e {
    if let actix_web::client::ConnectError::Timeout = e {
      return true;
    }
  }

  false
}
