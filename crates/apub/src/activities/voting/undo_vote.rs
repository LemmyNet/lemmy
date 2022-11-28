use crate::{
  activities::{
    generate_activity_id,
    verify_person_in_community,
    voting::{undo_vote_comment, undo_vote_post},
  },
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{
    activities::voting::{undo_vote::UndoVote, vote::Vote},
    InCommunity,
  },
  ActorType,
  PostOrComment,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  traits::ActivityHandler,
  utils::verify_urls_match,
};
use activitystreams_kinds::activity::UndoType;
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyError;
use url::Url;

impl UndoVote {
  pub(in crate::activities::voting) fn new(
    vote: Vote,
    actor: &ApubPerson,
    community: &ApubCommunity,
    context: &LemmyContext,
  ) -> Result<Self, LemmyError> {
    Ok(UndoVote {
      actor: ObjectId::new(actor.actor_id()),
      object: vote,
      kind: UndoType::Undo,
      id: generate_activity_id(
        UndoType::Undo,
        &context.settings().get_protocol_and_hostname(),
      )?,
      audience: Some(ObjectId::new(community.actor_id())),
    })
  }
}

#[async_trait::async_trait(?Send)]
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
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = self.community(context, request_counter).await?;
    verify_person_in_community(&self.actor, &community, context, request_counter).await?;
    verify_urls_match(self.actor.inner(), self.object.actor.inner())?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor = self
      .actor
      .dereference(context, local_instance(context).await, request_counter)
      .await?;
    let object = self
      .object
      .object
      .dereference(context, local_instance(context).await, request_counter)
      .await?;
    match object {
      PostOrComment::Post(p) => undo_vote_post(actor, &p, context).await,
      PostOrComment::Comment(c) => undo_vote_comment(actor, &c, context).await,
    }
  }
}
