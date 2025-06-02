use crate::{
  activities::{
    generate_activity_id,
    voting::{undo_vote_comment, undo_vote_post},
  },
  insert_received_activity,
  protocol::activities::voting::{undo_vote::UndoVote, vote::Vote},
};
use activitypub_federation::{
  config::Data,
  kinds::activity::UndoType,
  protocol::verification::verify_urls_match,
  traits::{ActivityHandler, Actor},
};
use lemmy_api_common::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{person::ApubPerson, PostOrComment},
  utils::{functions::verify_person_in_community, protocol::InCommunity},
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl UndoVote {
  pub(in crate::activities::voting) fn new(
    vote: Vote,
    actor: &ApubPerson,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<Self> {
    Ok(UndoVote {
      actor: actor.id().into(),
      object: vote,
      kind: UndoType::Undo,
      id: generate_activity_id(
        UndoType::Undo,
        &context.settings().get_protocol_and_hostname(),
      )?,
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

  async fn verify(&self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    let community = self.object.community(context).await?;
    verify_person_in_community(&self.actor, &community, context).await?;
    verify_urls_match(self.actor.inner(), self.object.actor.inner())?;
    self.object.verify(context).await?;
    Ok(())
  }

  async fn receive(self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    insert_received_activity(&self.id, context).await?;
    let actor = self.actor.dereference(context).await?;
    let object = self.object.object.dereference(context).await?;
    match object {
      PostOrComment::Left(p) => undo_vote_post(actor, &p, context).await,
      PostOrComment::Right(c) => undo_vote_comment(actor, &c, context).await,
    }
  }
}
