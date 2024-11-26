use crate::{
  objects::{community::ApubCommunity, note_wrapper::is_public},
  protocol::{
    activities::create_or_update::{
      note::CreateOrUpdateNote,
      note_wrapper::CreateOrUpdateNoteWrapper,
      private_message::CreateOrUpdatePrivateMessage,
    },
    InCommunity,
  },
};
use activitypub_federation::{config::Data, traits::ActivityHandler};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::{FederationError, LemmyError, LemmyResult};
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
  async fn verify(&self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let val = to_value(self)?;
    if is_public(&self.to, &self.cc) {
      CreateOrUpdateNote::verify(&from_value(val)?, context).await?;
    } else {
      CreateOrUpdatePrivateMessage::verify(&from_value(val)?, context).await?;
    }
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let is_public = is_public(&self.to, &self.cc);
    let val = to_value(self)?;
    if is_public {
      CreateOrUpdateNote::receive(from_value(val)?, context).await?;
    } else {
      CreateOrUpdatePrivateMessage::receive(from_value(val)?, context).await?;
    }
    Ok(())
  }
}

#[async_trait::async_trait]
impl InCommunity for CreateOrUpdateNoteWrapper {
  #[tracing::instrument(skip(self, context))]
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    if is_public(&self.to, &self.cc) {
      let comment: CreateOrUpdateNote = from_value(to_value(self)?)?;
      comment.community(context).await
    } else {
      Err(FederationError::ObjectIsNotPublic.into())
    }
  }
}
