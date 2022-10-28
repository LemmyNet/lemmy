use crate::{local_instance, ActorType};
use activitypub_federation::{core::object_id::ObjectId, traits::ApubObject};
use anyhow::anyhow;
use itertools::Itertools;
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{newtypes::DbUrl, source::local_site::LocalSite};
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use tracing::debug;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct WebfingerLink {
  pub rel: Option<String>,
  #[serde(rename = "type")]
  pub kind: Option<String>,
  pub href: Option<Url>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebfingerResponse {
  pub subject: String,
  pub links: Vec<WebfingerLink>,
}

/// Turns a person id like `@name@example.com` into an apub ID, like `https://example.com/user/name`,
/// using webfinger.
#[tracing::instrument(skip_all)]
pub(crate) async fn webfinger_resolve_actor<Kind>(
  identifier: &str,
  local_only: bool,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<DbUrl, LemmyError>
where
  Kind: ApubObject<DataType = LemmyContext, Error = LemmyError> + ActorType + Send + 'static,
  for<'de2> <Kind as ApubObject>::ApubType: serde::Deserialize<'de2>,
{
  let protocol = context.settings().get_protocol_string();
  let (_, domain) = identifier
    .splitn(2, '@')
    .collect_tuple()
    .ok_or_else(|| LemmyError::from_message("Invalid webfinger query, missing domain"))?;
  let fetch_url = format!(
    "{}://{}/.well-known/webfinger?resource=acct:{}",
    protocol, domain, identifier
  );
  debug!("Fetching webfinger url: {}", &fetch_url);

  let local_site = blocking(context.pool(), LocalSite::read).await?;
  let http_fetch_retry_limit = local_site
    .as_ref()
    .map(|l| l.federation_http_fetch_retry_limit)
    .unwrap_or(25);

  *request_counter += 1;
  if *request_counter > http_fetch_retry_limit {
    return Err(LemmyError::from_message("Request retry limit reached"));
  }

  let response = context.client().get(&fetch_url).send().await?;

  let res: WebfingerResponse = response.json().await.map_err(LemmyError::from)?;

  let links: Vec<Url> = res
    .links
    .iter()
    .filter(|link| {
      if let Some(type_) = &link.kind {
        type_.starts_with("application/")
      } else {
        false
      }
    })
    .filter_map(|l| l.href.clone())
    .collect();
  for l in links {
    let object_id = ObjectId::<Kind>::new(l);
    let object = if local_only {
      object_id.dereference_local(context).await
    } else {
      object_id
        .dereference(context, local_instance(context), request_counter)
        .await
    };
    if object.is_ok() {
      return object.map(|o| o.actor_id().into());
    }
  }
  let err = anyhow!("Failed to resolve actor for {}", identifier);
  Err(LemmyError::from_error_message(err, "failed_to_resolve"))
}
