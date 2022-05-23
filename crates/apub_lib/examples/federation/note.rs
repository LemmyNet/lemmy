use crate::person::MyUser;
use activitystreams_kinds::{object::NoteType, public};
use chrono::NaiveDateTime;
use lemmy_apub_lib::{object_id::ObjectId, traits::ApubObject};
use lemmy_utils::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, new)]
pub struct MyPost {
  pub text: String,
  pub ap_id: ObjectId<MyPost>,
  pub creator: ObjectId<MyUser>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
  #[serde(rename = "type")]
  kind: NoteType,
  id: ObjectId<MyPost>,
  attributed_to: ObjectId<MyUser>,
  to: Vec<Url>,
  content: String,
}

#[async_trait::async_trait(?Send)]
impl ApubObject for MyPost {
  type DataType = ();
  type ApubType = Note;
  type DbType = ();
  type TombstoneType = ();

  async fn read_from_apub_id(
    object_id: Url,
    data: &Self::DataType,
  ) -> Result<Option<Self>, LemmyError> {
    Ok(None)
  }

  async fn delete(self, data: &Self::DataType) -> Result<(), LemmyError> {
    todo!()
  }

  async fn into_apub(self, data: &Self::DataType) -> Result<Self::ApubType, LemmyError> {
    let creator = self.creator.dereference_local(data).await?;
    Ok(Note {
      kind: Default::default(),
      id: self.ap_id,
      attributed_to: self.creator,
      to: vec![public(), creator.followers()?],
      content: self.text,
    })
  }

  fn to_tombstone(&self) -> Result<Self::TombstoneType, LemmyError> {
    todo!()
  }

  async fn verify(
    apub: &Self::ApubType,
    expected_domain: &Url,
    data: &Self::DataType,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    todo!()
  }

  async fn from_apub(
    apub: Self::ApubType,
    data: &Self::DataType,
    request_counter: &mut i32,
  ) -> Result<Self, LemmyError> {
    Ok(MyPost {
      text: apub.content,
      ap_id: apub.id,
      creator: apub.attributed_to,
    })
  }
}
