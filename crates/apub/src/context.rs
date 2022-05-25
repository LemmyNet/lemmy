use lemmy_apub_lib::{data::Data, traits::ActivityHandler};
use lemmy_utils::LemmyError;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use url::Url;

static CONTEXT: Lazy<Vec<serde_json::Value>> = Lazy::new(|| {
  serde_json::from_str(include_str!("../assets/lemmy/context.json")).expect("parse context")
});

#[derive(Serialize, Deserialize, Debug)]
pub struct WithContext<T> {
  #[serde(rename = "@context")]
  #[serde(deserialize_with = "crate::deserialize_one_or_many")]
  context: Vec<serde_json::Value>,
  #[serde(flatten)]
  inner: T,
}

impl<T> WithContext<T> {
  pub(crate) fn new(inner: T) -> WithContext<T> {
    WithContext {
      context: (*CONTEXT).clone(),
      inner,
    }
  }
}

#[async_trait::async_trait(?Send)]
impl<T> ActivityHandler for WithContext<T>
where
  T: ActivityHandler,
{
  type DataType = <T as ActivityHandler>::DataType;

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
  ) -> Result<(), LemmyError> {
    self.inner.verify(data, request_counter).await
  }

  async fn receive(
    self,
    data: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    self.inner.receive(data, request_counter).await
  }
}
