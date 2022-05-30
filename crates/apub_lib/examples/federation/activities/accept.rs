use crate::{activities::follow::Follow, objects::person::MyUser};
use activitystreams_kinds::activity::AcceptType;
use lemmy_apub_lib::{data::Data, object_id::ObjectId, traits::ActivityHandler};
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Accept {
  actor: ObjectId<MyUser>,
  object: Follow,
  #[serde(rename = "type")]
  kind: AcceptType,
  id: Url,
}

impl Accept {
  pub fn new(actor: ObjectId<MyUser>, object: Follow, id: Url) -> Accept {
    Accept {
      actor,
      object,
      kind: Default::default(),
      id,
    }
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for Accept {
  type DataType = ();

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(
    &self,
    _data: &Data<Self::DataType>,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    todo!()
  }

  async fn receive(
    self,
    _data: &Data<Self::DataType>,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    todo!()
  }
}
