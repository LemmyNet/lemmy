use crate::{
  objects::community::ApubCommunity,
  protocol::{
    activities::create_or_update::{
      note::CreateOrUpdateNote,
      note_wrapper::CreateOrUpdateNoteWrapper,
      private_message::CreateOrUpdatePrivateMessage,
    },
    InCommunity,
  },
};
use activitypub_federation::{config::Data, kinds::public, traits::ActivityHandler};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::{LemmyError, LemmyResult};
use serde_json::{from_value, to_value};
use url::Url;

#[async_trait::async_trait]
impl ActivityHandler for CreateOrUpdateNoteWrapper {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    &self.actor
  }

  #[tracing::instrument(skip_all)]
  async fn verify(&self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
    // Do everything in receive to avoid extra checks.
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let val = to_value(self)?;
    let comment: CreateOrUpdateNote = from_value(val.clone())?;
    // First check if the activity is marked as public which is true for all comments
    // in public communities. However that check doesnt work in private communities. So if
    // it fails we need to resolve the community which is much slower as it requires
    // db reads and maybe network fetches.
    if comment.cc.contains(&public())
      || comment.to.contains(&public())
      || comment.community(context).await.is_ok()
    {
      CreateOrUpdateNote::verify(&comment, context).await?;
      CreateOrUpdateNote::receive(comment, context).await?;
    } else {
      let private_message = from_value(val)?;
      CreateOrUpdatePrivateMessage::verify(&private_message, context).await?;
      CreateOrUpdatePrivateMessage::receive(private_message, context).await?;
    }
    Ok(())
  }
}

#[async_trait::async_trait]
impl InCommunity for CreateOrUpdateNoteWrapper {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let val = to_value(self)?;
    let comment: CreateOrUpdateNote = from_value(val.clone())?;
    comment.community(context).await
  }
}
