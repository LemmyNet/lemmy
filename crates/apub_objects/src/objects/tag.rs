use activitypub_federation::{config::Data, traits::Object};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::tag::Tag;
use lemmy_utils::error::{LemmyError, LemmyResult};
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubTag(pub Tag);

impl Deref for ApubTag {
  type Target = Tag;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[async_trait::async_trait]
impl Object for ApubTag {
  type DataType = LemmyContext;
  type Kind = Tag;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    self.ap_id.inner()
  }

  async fn read_from_id(
    object_id: Url,
    context: &Data<Self::DataType>,
  ) -> LemmyResult<Option<Self>> {
    todo!()
  }

  async fn into_json(self, context: &Data<Self::DataType>) -> LemmyResult<Self::Kind> {
    todo!()
  }

  async fn verify(
    note: &Tag,
    expected_domain: &Url,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    todo!()
  }

  async fn from_json(note: Self::Kind, context: &Data<LemmyContext>) -> LemmyResult<Self> {
    todo!()
  }
}
