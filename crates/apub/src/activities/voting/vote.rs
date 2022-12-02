use crate::{
  activities::{
    generate_activity_id,
    verify_person_in_community,
    voting::{vote_comment, vote_post},
  },
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{
    activities::voting::vote::{Vote, VoteType},
    InCommunity,
  },
  ActorType,
  PostOrComment,
};
use activitypub_federation::{core::object_id::ObjectId, data::Data, traits::ActivityHandler};
use anyhow::anyhow;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::local_site::LocalSite;
use lemmy_utils::error::LemmyError;
use url::Url;

impl Vote {
  pub(in crate::activities::voting) fn new(
    object_id: ObjectId<PostOrComment>,
    actor: &ApubPerson,
    community: &ApubCommunity,
    kind: VoteType,
    context: &LemmyContext,
  ) -> Result<Vote, LemmyError> {
    Ok(Vote {
      actor: ObjectId::new(actor.actor_id()),
      object: object_id,
      kind: kind.clone(),
      id: generate_activity_id(kind, &context.settings().get_protocol_and_hostname())?,
      audience: Some(ObjectId::new(community.actor_id())),
    })
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for Vote {
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
    let enable_downvotes = LocalSite::read(context.pool())
      .await
      .map(|l| l.enable_downvotes)
      .unwrap_or(true);
    if self.kind == VoteType::Dislike && !enable_downvotes {
      return Err(anyhow!("Downvotes disabled").into());
    }
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
      .dereference(context, local_instance(context).await, request_counter)
      .await?;
    match object {
      PostOrComment::Post(p) => vote_post(&self.kind, actor, &p, context).await,
      PostOrComment::Comment(c) => vote_comment(&self.kind, actor, &c, context).await,
    }
  }
}
