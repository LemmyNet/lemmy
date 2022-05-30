use crate::{activities::accept::Accept, generate_object_id, objects::person::MyUser};
use activitystreams_kinds::activity::FollowType;
use lemmy_apub_lib::{data::Data, object_id::ObjectId, traits::ActivityHandler};
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize)]
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
    data: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let target = self.object.dereference(data, request_counter).await?;

    let id = generate_object_id(hostname)?;
    let follow = Accept::new(self.object, self.clone(), id.clone());
    target
      .send(
        id,
        follow,
        vec![other.ap_id.clone().into_inner()],
        local_instance,
      )
      .await?;
    Ok(())
  }
}
