use crate::{data::Data, deser::deserialize_one_or_many, traits::ActivityHandler};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use url::Url;

const DEFAULT_CONTEXT: &str = "[\"https://www.w3.org/ns/activitystreams\"]";

/// Simple wrapper which adds json-ld context to an object or activity. Doing it this way ensures
/// that nested objects dont have any context, but only the outermost one.
#[derive(Serialize, Deserialize, Debug)]
pub struct WithContext<T> {
  #[serde(rename = "@context")]
  #[serde(deserialize_with = "deserialize_one_or_many")]
  context: Vec<Value>,
  #[serde(flatten)]
  inner: T,
}

impl<T> WithContext<T> {
  pub fn new_default(inner: T) -> WithContext<T> {
    let context = vec![Value::from_str(DEFAULT_CONTEXT).expect("valid context")];
    WithContext::new(inner, context)
  }

  pub fn new(inner: T, context: Vec<Value>) -> WithContext<T> {
    WithContext { context, inner }
  }
}

#[async_trait::async_trait(?Send)]
impl<T> ActivityHandler for WithContext<T>
where
  T: ActivityHandler,
{
  type DataType = <T as ActivityHandler>::DataType;
  type Error = <T as ActivityHandler>::Error;

  fn id(&self) -> &Url {
    self.inner.id()
  }

  fn actor(&self) -> &Url {
    self.inner.actor()
  }

  async fn verify(
    &self,
    data: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), Self::Error> {
    self.inner.verify(data, request_counter).await
  }

  async fn receive(
    self,
    data: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), Self::Error> {
    self.inner.receive(data, request_counter).await
  }
}
