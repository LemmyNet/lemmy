use crate::objects::community::ApubCommunity;
use activitypub_federation::{
  config::Data,
  fetch::fetch_object_http,
  kinds::object::ImageType,
  protocol::values::MediaTypeMarkdown,
};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::newtypes::DbUrl;
use lemmy_utils::error::LemmyResult;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, future::Future};
use url::Url;

pub mod activities;
pub(crate) mod collections;
pub(crate) mod objects;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Unparsed(HashMap<String, serde_json::Value>);


#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum IdOrNestedObject<Kind: Id> {
  Id(Url),
  NestedObject(Kind),
}

impl<Kind: Id + DeserializeOwned + Send> IdOrNestedObject<Kind> {
  pub(crate) fn id(&self) -> &Url {
    match self {
      IdOrNestedObject::Id(i) => i,
      IdOrNestedObject::NestedObject(n) => n.object_id(),
    }
  }
  pub(crate) async fn object(self, context: &Data<LemmyContext>) -> LemmyResult<Kind> {
    match self {
      // TODO: move IdOrNestedObject struct to library and make fetch_object_http private
      IdOrNestedObject::Id(i) => Ok(fetch_object_http(&i, context).await?.object),
      IdOrNestedObject::NestedObject(o) => Ok(o),
    }
  }
}
