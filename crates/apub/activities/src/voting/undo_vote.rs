use crate::{
  generate_activity_id,
  protocol::voting::{undo_vote::UndoVote, vote::Vote},
  voting::{undo_vote_comment, undo_vote_post},
};
use activitypub_federation::{
  config::Data,
  kinds::activity::UndoType,
  protocol::verification::verify_urls_match,
  traits::{Activity, Object},
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{PostOrComment, person::ApubPerson},
  utils::{functions::verify_person_in_community, protocol::InCommunity},
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl UndoVote {
  pub(in crate::voting) fn new(
    vote: Vote,
    actor: &ApubPerson,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<Self> {
    Ok(UndoVote {
      actor: actor.id().clone().into(),
      object: vote,
      kind: UndoType::Undo,
      id: generate_activity_id(UndoType::Undo, context)?,
    })
  }
}

#[async_trait::async_trait]
impl Activity for UndoVote {
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
    let actor = self.actor.dereference(context).await?;
    let object = self.object.object.dereference(context).await?;
    match object {
      PostOrComment::Left(p) => undo_vote_post(actor, &p, context).await,
      PostOrComment::Right(c) => undo_vote_comment(actor, &c, context).await,
    }
  }
}
