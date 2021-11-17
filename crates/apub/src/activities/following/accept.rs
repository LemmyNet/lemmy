use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_activity},
  protocol::activities::following::{accept::AcceptFollowCommunity, follow::FollowCommunity},
};
use activitystreams::activity::kind::AcceptType;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  traits::{ActivityHandler, ActorType},
  verify::verify_urls_match,
};
use lemmy_db_schema::{source::community::CommunityFollower, traits::Followable};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;

impl AcceptFollowCommunity {
  pub async fn send(
    follow: FollowCommunity,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = follow.object.dereference_local(context).await?;
    let person = follow
      .actor
      .clone()
      .dereference(context, request_counter)
      .await?;
    let accept = AcceptFollowCommunity {
      actor: ObjectId::new(community.actor_id()),
      object: follow,
      kind: AcceptType::Accept,
      id: generate_activity_id(
        AcceptType::Accept,
        &context.settings().get_protocol_and_hostname(),
      )?,
      unparsed: Default::default(),
    };
    let inbox = vec![person.inbox_url()];
    send_lemmy_activity(context, &accept, &accept.id, &community, inbox, true).await
  }
}

/// Handle accepted follows
#[async_trait::async_trait(?Send)]
impl ActivityHandler for AcceptFollowCommunity {
  type DataType = LemmyContext;
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(&self.id, self.actor.inner(), &context.settings())?;
    verify_urls_match(self.actor.inner(), self.object.object.inner())?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let person = self.actor.dereference(context, request_counter).await?;
    let community = self
      .object
      .actor
      .dereference(context, request_counter)
      .await?;
    // This will throw an error if no follow was requested
    blocking(context.pool(), move |conn| {
      CommunityFollower::follow_accepted(conn, person.id, community.id)
    })
    .await??;

    Ok(())
  }
}
