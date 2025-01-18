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
use activitypub_federation::{config::Data, traits::ActivityHandler};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::{LemmyError, LemmyResult};
use serde_json::{from_value, to_value};
use url::Url;

/// In Activitypub, both private messages and comments are represented by `type: Note` which
/// makes it difficult to distinguish them. This wrapper handles receiving of both types, and
/// routes them to the correct handler.
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
    // Use serde to convert NoteWrapper either into Comment or PrivateMessage,
    // depending on conditions below. This works because NoteWrapper keeps all
    // additional data in field `other: Map<String, Value>`.
    let val = to_value(self)?;

    // Convert self to a comment and get the community. If the conversion is
    // successful and a community is returned, this is a comment.
    let comment = from_value::<CreateOrUpdateNote>(val.clone());
    if let Ok(comment) = comment {
      if comment.community(context).await.is_ok() {
        CreateOrUpdateNote::verify(&comment, context).await?;
        CreateOrUpdateNote::receive(comment, context).await?;
        return Ok(());
      }
    }

    // If any of the previous checks failed, we are dealing with a private message.
    let private_message = from_value(val)?;
    CreateOrUpdatePrivateMessage::verify(&private_message, context).await?;
    CreateOrUpdatePrivateMessage::receive(private_message, context).await?;
    Ok(())
  }
}

#[async_trait::async_trait]
impl InCommunity for CreateOrUpdateNoteWrapper {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    // Same logic as in receive. In case this is a private message, an error is returned.
    let val = to_value(self)?;
    let comment: CreateOrUpdateNote = from_value(val.clone())?;
    comment.community(context).await
  }
}
