use crate::{generate_object_id, objects::person::MyUser};
use activitystreams_kinds::{object::NoteType, public};
use lemmy_apub_lib::{deser::deserialize_one_or_many, object_id::ObjectId, traits::ApubObject};
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone)]
pub struct MyPost {
  pub text: String,
  pub ap_id: ObjectId<MyPost>,
  pub creator: ObjectId<MyUser>,
  pub local: bool,
}

impl MyPost {
  pub fn new(text: String, creator: ObjectId<MyUser>) -> MyPost {
    MyPost {
      text,
      ap_id: ObjectId::new(generate_object_id(creator.inner().domain().unwrap()).unwrap()),
      creator,
      local: true,
    }
  }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
  #[serde(rename = "type")]
  kind: NoteType,
  id: ObjectId<MyPost>,
  pub(crate) attributed_to: ObjectId<MyUser>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  content: String,
}

#[async_trait::async_trait(?Send)]
impl ApubObject for MyPost {
  type DataType = ();
  type ApubType = Note;
  type DbType = ();
  type TombstoneType = ();

  async fn read_from_apub_id(
    _object_id: Url,
    _data: &Self::DataType,
  ) -> Result<Option<Self>, LemmyError> {
    Ok(None)
  }

  async fn delete(self, _data: &Self::DataType) -> Result<(), LemmyError> {
    todo!()
  }

  async fn into_apub(self, data: &Self::DataType) -> Result<Self::ApubType, LemmyError> {
    let creator = self.creator.dereference_local(data).await?;
    Ok(Note {
      kind: Default::default(),
      id: self.ap_id,
      attributed_to: self.creator,
      to: vec![public(), creator.followers_url()?],
      content: self.text,
    })
  }

  fn to_tombstone(&self) -> Result<Self::TombstoneType, LemmyError> {
    todo!()
  }

  async fn verify(
    _apub: &Self::ApubType,
    _expected_domain: &Url,
    _data: &Self::DataType,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    todo!()
  }

  async fn from_apub(
    apub: Self::ApubType,
    _data: &Self::DataType,
    _request_counter: &mut i32,
  ) -> Result<Self, LemmyError> {
    Ok(MyPost {
      text: apub.content,
      ap_id: apub.id,
      creator: apub.attributed_to,
      local: false,
    })
  }
}
