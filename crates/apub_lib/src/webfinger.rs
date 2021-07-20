use anyhow::anyhow;
use lemmy_utils::{
  request::{retry, RecvError},
  settings::structs::Settings,
  LemmyError,
};
use log::debug;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct WebfingerLink {
  pub rel: Option<String>,
  #[serde(rename(serialize = "type", deserialize = "type"))]
  pub type_: Option<String>,
  pub href: Option<Url>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub template: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebfingerResponse {
  pub subject: String,
  pub aliases: Vec<Url>,
  pub links: Vec<WebfingerLink>,
}

pub enum WebfingerType {
  Person,
  Group,
}

/// Turns a person id like `@name@example.com` into an apub ID, like `https://example.com/user/name`,
/// using webfinger.
pub async fn webfinger_resolve_actor(
  name: &str,
  domain: &str,
  webfinger_type: WebfingerType,
  client: &Client,
) -> Result<Url, LemmyError> {
  let webfinger_type = match webfinger_type {
    WebfingerType::Person => "acct",
    WebfingerType::Group => "group",
  };
  let fetch_url = format!(
    "{}://{}/.well-known/webfinger?resource={}:{}@{}",
    Settings::get().get_protocol_string(),
    domain,
    webfinger_type,
    name,
    domain
  );
  debug!("Fetching webfinger url: {}", &fetch_url);

  let response = retry(|| client.get(&fetch_url).send()).await?;

  let res: WebfingerResponse = response
    .json()
    .await
    .map_err(|e| RecvError(e.to_string()))?;

  let link = res
    .links
    .iter()
    .find(|l| l.type_.eq(&Some("application/activity+json".to_string())))
    .ok_or_else(|| anyhow!("No application/activity+json link found."))?;
  link
    .href
    .to_owned()
    .ok_or_else(|| anyhow!("No href found.").into())
}
