use crate::{
  objects::{community::ApubCommunity, instance::ApubSite, person::ApubPerson},
  protocol::{group::Group, instance::Instance},
};
use activitypub_federation::{config::Data, protocol::context::WithContext, traits::Object};
use assert_json_diff::assert_json_include;
use lemmy_api_utils::context::LemmyContext;
use lemmy_utils::error::LemmyResult;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, fs::File, io::BufReader};
use url::Url;

pub fn file_to_json_object<T: DeserializeOwned>(path: &str) -> LemmyResult<T> {
  let file = File::open(path)?;
  let reader = BufReader::new(file);
  Ok(serde_json::from_reader(reader)?)
}

pub fn test_json<T: DeserializeOwned>(path: &str) -> LemmyResult<WithContext<T>> {
  file_to_json_object::<WithContext<T>>(path)
}

/// Check that json deserialize -> serialize -> deserialize gives identical file as initial one.
/// Ensures that there are no breaking changes in sent data.
pub fn test_parse_lemmy_item<T: Serialize + DeserializeOwned + std::fmt::Debug>(
  path: &str,
) -> LemmyResult<T> {
  // parse file as T
  let parsed = file_to_json_object::<T>(path)?;

  // parse file into hashmap, which ensures that every field is included
  let raw = file_to_json_object::<HashMap<String, serde_json::Value>>(path)?;
  // assert that all fields are identical, otherwise print diff
  assert_json_include!(actual: &parsed, expected: raw);
  Ok(parsed)
}

pub(crate) async fn parse_lemmy_instance(context: &Data<LemmyContext>) -> LemmyResult<ApubSite> {
  let json: Instance = file_to_json_object("../apub/assets/lemmy/objects/instance.json")?;
  let id = Url::parse("https://enterprise.lemmy.ml/")?;
  ApubSite::verify(&json, &id, context).await?;
  let site = ApubSite::from_json(json, context).await?;
  assert_eq!(context.request_count(), 0);
  Ok(site)
}

pub async fn parse_lemmy_person(
  context: &Data<LemmyContext>,
) -> LemmyResult<(ApubPerson, ApubSite)> {
  let site = parse_lemmy_instance(context).await?;
  let json = file_to_json_object("../apub/assets/lemmy/objects/person.json")?;
  let url = Url::parse("https://enterprise.lemmy.ml/u/picard")?;
  ApubPerson::verify(&json, &url, context).await?;
  let person = ApubPerson::from_json(json, context).await?;
  assert_eq!(context.request_count(), 0);
  Ok((person, site))
}

pub async fn parse_lemmy_community(context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
  // use separate counter so this doesn't affect tests
  let context2 = context.clone();
  let mut json: Group = file_to_json_object("../apub/assets/lemmy/objects/group.json")?;
  // change these links so they dont fetch over the network
  json.attributed_to = None;
  json.outbox = Url::parse("https://enterprise.lemmy.ml/c/tenforward/not_outbox")?;
  json.followers = Some(Url::parse(
    "https://enterprise.lemmy.ml/c/tenforward/not_followers",
  )?);

  let url = Url::parse("https://enterprise.lemmy.ml/c/tenforward")?;
  ApubCommunity::verify(&json, &url, &context2).await?;
  let community = ApubCommunity::from_json(json, &context2).await?;
  Ok(community)
}
