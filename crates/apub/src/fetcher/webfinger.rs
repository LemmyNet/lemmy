use crate::{local_instance, ActorType, FEDERATION_HTTP_FETCH_LIMIT};
use activitypub_federation::{core::object_id::ObjectId, traits::ApubObject};
use anyhow::anyhow;
use itertools::Itertools;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::newtypes::DbUrl;
use lemmy_utils::{error::LemmyError, WebfingerResponse};
use tracing::debug;
use url::Url;

/// Turns a person id like `@name@example.com` into an apub ID, like `https://example.com/user/name`,
/// using webfinger.
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
  let fetch_url = format!("{protocol}://{domain}/.well-known/webfinger?resource=acct:{identifier}");
  debug!("Fetching webfinger url: {}", &fetch_url);

  *request_counter += 1;
  if *request_counter > FEDERATION_HTTP_FETCH_LIMIT {
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
        .dereference(context, local_instance(context).await, request_counter)
        .await
    };
    if object.is_ok() {
      return object.map(|o| o.actor_id().into());
    }
  }
  let err = anyhow!("Failed to resolve actor for {}", identifier);
  Err(LemmyError::from_error_message(err, "failed_to_resolve"))
}
