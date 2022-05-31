use crate::{objects::person::MyUser, ObjectId};
use activitypub_federation::{data::Data, traits::ActivityHandler};
use activitystreams_kinds::activity::FollowType;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
  pub(crate) actor: ObjectId<MyUser>,
  pub(crate) object: ObjectId<MyUser>,
  #[serde(rename = "type")]
  kind: FollowType,
  id: Url,
}

impl Follow {
  pub fn new(actor: ObjectId<MyUser>, object: ObjectId<MyUser>, id: Url) -> Follow {
    Follow {
      actor,
      object,
      kind: Default::default(),
      id,
    }
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for Follow {
  type DataType = ();
  type Error = crate::error::Error;

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
  ) -> Result<(), Self::Error> {
    todo!()
  }

  async fn receive(
    self,
    _data: &Data<Self::DataType>,
    _request_counter: &mut i32,
  ) -> Result<(), Self::Error> {
    todo!()
  }
}
