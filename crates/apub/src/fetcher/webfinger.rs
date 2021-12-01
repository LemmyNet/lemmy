use crate::{generate_local_apub_endpoint, EndpointType};
use anyhow::anyhow;
use itertools::Itertools;
use lemmy_apub_lib::{
  object_id::ObjectId,
  traits::{ActorType, ApubObject},
};
use lemmy_db_schema::newtypes::DbUrl;
use lemmy_utils::{
  request::{retry, RecvError, RETRY_LIMIT},
  LemmyError,
};
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

/// Takes in a shortname of the type dessalines@xyz.tld or dessalines (assumed to be local), and
/// outputs the actor id. Used in the API for communities and users.
///
/// TODO: later provide a method in ApubObject to generate the endpoint, so that we dont have to
///       pass in EndpointType
pub async fn webfinger_resolve<Kind>(
  identifier: &str,
  endpoint_type: EndpointType,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<DbUrl, LemmyError>
where
  Kind: ApubObject<DataType = LemmyContext> + ActorType + Send + 'static,
  for<'de2> <Kind as ApubObject>::ApubType: serde::Deserialize<'de2>,
{
  // remote actor
  if identifier.contains('@') {
    webfinger_resolve_actor::<Kind>(identifier, context, request_counter).await
  }
  // local actor
  else {
    let domain = context.settings().get_protocol_and_hostname();
    Ok(generate_local_apub_endpoint(
      endpoint_type,
      identifier,
      &domain,
    )?)
  }
}

/// Turns a person id like `@name@example.com` into an apub ID, like `https://example.com/user/name`,
/// using webfinger.
pub(crate) async fn webfinger_resolve_actor<Kind>(
  identifier: &str,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<DbUrl, LemmyError>
where
  Kind: ApubObject<DataType = LemmyContext> + ActorType + Send + 'static,
  for<'de2> <Kind as ApubObject>::ApubType: serde::Deserialize<'de2>,
{
  let protocol = context.settings().get_protocol_string();
  let (_, domain) = identifier
    .splitn(2, '@')
    .collect_tuple()
    .expect("invalid query");
  let fetch_url = format!(
    "{}://{}/.well-known/webfinger?resource=acct:{}",
    protocol, domain, identifier
  );
  debug!("Fetching webfinger url: {}", &fetch_url);

  *request_counter += 1;
  if *request_counter > RETRY_LIMIT {
    return Err(LemmyError::from(anyhow!("Request retry limit reached")));
  }

  let response = retry(|| context.client().get(&fetch_url).send()).await?;

  let res: WebfingerResponse = response
    .json()
    .await
    .map_err(|e| RecvError(e.to_string()))?;

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
    .map(|l| l.href.clone())
    .flatten()
    .collect();
  for l in links {
    let object = ObjectId::<Kind>::new(l)
      .dereference(context, request_counter)
      .await;
    if object.is_ok() {
      return object.map(|o| o.actor_id().into());
    }
  }
  Err(anyhow!("Failed to resolve actor for {}", identifier).into())
}
