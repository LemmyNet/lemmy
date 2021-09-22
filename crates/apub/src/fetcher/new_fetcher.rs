use crate::{fetcher::should_refetch_actor, objects::FromApub, APUB_JSON_CONTENT_TYPE};
use anyhow::anyhow;
use diesel::NotFound;
use lemmy_api_common::blocking;
use lemmy_db_queries::{ApubObject, DbPool};
use lemmy_utils::{request::retry, settings::structs::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use log::debug;
use reqwest::StatusCode;
use std::time::Duration;
use url::Url;

/// Maximum number of HTTP requests allowed to handle a single incoming activity (or a single object
/// fetch through the search). This should be configurable.
static REQUEST_LIMIT: i32 = 25;

/// Fetches an activitypub object, either from local database (if possible), or over http.
pub(crate) async fn dereference<Kind>(
  id: &Url,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<Kind, LemmyError>
where
  Kind: FromApub + ApubObject + Send + 'static,
  for<'de> <Kind as FromApub>::ApubType: serde::Deserialize<'de>,
{
  let db_object = dereference_locally::<Kind>(id.clone(), context.pool()).await?;
  // if its a local object, only fetch it from the database and not over http
  if id.domain() == Some(&Settings::get().get_hostname_without_port()?) {
    return match db_object {
      None => Err(NotFound {}.into()),
      Some(o) => Ok(o),
    };
  }

  if let Some(object) = db_object {
    if let Some(last_refreshed_at) = object.last_refreshed_at() {
      // TODO: rename to should_refetch_object()
      if should_refetch_actor(last_refreshed_at) {
        debug!("Refetching remote object {}", id);
        return dereference_remotely(id, context, request_counter).await;
      }
    }
    Ok(object)
  } else {
    debug!("Fetching remote object {}", id);
    dereference_remotely(id, context, request_counter).await
  }
}

/// returning none means the object was not found in local db
async fn dereference_locally<Kind>(id: Url, pool: &DbPool) -> Result<Option<Kind>, LemmyError>
where
  Kind: FromApub + ApubObject + Send + 'static,
  for<'de> <Kind as FromApub>::ApubType: serde::Deserialize<'de>,
{
  let object = blocking(pool, move |conn| {
    ApubObject::read_from_apub_id(conn, &id.into())
  })
  .await?;
  match object {
    Ok(o) => Ok(Some(o)),
    Err(NotFound {}) => Ok(None),
    Err(e) => Err(e.into()),
  }
}

async fn dereference_remotely<Kind>(
  id: &Url,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<Kind, LemmyError>
where
  Kind: FromApub + ApubObject + Send + 'static,
  for<'de> <Kind as FromApub>::ApubType: serde::Deserialize<'de>,
{
  // dont fetch local objects this way
  debug_assert!(id.domain() != Some(&Settings::get().hostname));

  *request_counter += 1;
  if *request_counter > REQUEST_LIMIT {
    return Err(LemmyError::from(anyhow!("Request limit reached")));
  }

  let res = retry(|| {
    context
      .client()
      .get(id.as_str())
      .header("Accept", APUB_JSON_CONTENT_TYPE)
      .timeout(Duration::from_secs(60))
      .send()
  })
  .await?;

  if res.status() == StatusCode::GONE {
    mark_object_deleted::<Kind>(id, context).await?;
    return Err(anyhow!("Fetched remote object {} which was deleted", id.to_string()).into());
  }

  let res2: Kind::ApubType = res.json().await?;

  Ok(Kind::from_apub(&res2, context, id, request_counter).await?)
}

async fn mark_object_deleted<Kind>(_id: &Url, _context: &LemmyContext) -> Result<(), LemmyError>
where
  Kind: FromApub + ApubObject + Send + 'static,
{
  // TODO: need to move methods update_deleted, update_removed etc into a trait to use them here.
  //       also, how do we know if the object was deleted vs removed?
  todo!()
}
