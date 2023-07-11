use crate::{
  activities::{
    generate_activity_id,
    verify_person_in_community,
    voting::{undo_vote_comment, undo_vote_post},
  },
  insert_received_activity,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{
    activities::voting::{undo_vote::UndoVote, vote::Vote},
    InCommunity,
  },
  PostOrComment,
};
use activitypub_federation::{
  config::Data,
  kinds::activity::UndoType,
  protocol::verification::verify_urls_match,
  traits::{ActivityHandler, Actor},
};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyError;
use url::Url;

impl UndoVote {
  pub(in crate::activities::voting) fn new(
    vote: Vote,
    actor: &ApubPerson,
    community: &ApubCommunity,
    context: &Data<LemmyContext>,
  ) -> Result<Self, LemmyError> {
    Ok(UndoVote {
      actor: actor.id().into(),
      object: vote,
      kind: UndoType::Undo,
      id: generate_activity_id(
        UndoType::Undo,
        &context.settings().get_protocol_and_hostname(),
      )?,
      audience: Some(community.id().into()),
    })
  }
}

#[async_trait::async_trait]
impl ActivityHandler for UndoVote {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(&self, context: &Data<LemmyContext>) -> Result<(), LemmyError> {
    let community = self.community(context).await?;
    verify_person_in_community(&self.actor, &community, context).await?;
    verify_urls_match(self.actor.inner(), self.object.actor.inner())?;
    self.object.verify(context).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<LemmyContext>) -> Result<(), LemmyError> {
    insert_received_activity(&self.id, context).await?;
    let actor = self.actor.dereference(context).await?;
    let object = self.object.object.dereference(context).await?;
    match object {
      PostOrComment::Post(p) => undo_vote_post(actor, &p, context).await,
      PostOrComment::Comment(c) => undo_vote_comment(actor, &c, context).await,
    }
  }
}
