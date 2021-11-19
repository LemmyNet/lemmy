use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_activity, verify_person},
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::following::{follow::FollowCommunity, undo_follow::UndoFollowCommunity},
};
use activitystreams_kinds::activity::UndoType;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  traits::{ActivityHandler, ActorType},
  verify::verify_urls_match,
};
use lemmy_db_schema::{
  source::community::{CommunityFollower, CommunityFollowerForm},
  traits::Followable,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;

impl UndoFollowCommunity {
  pub async fn send(
    actor: &ApubPerson,
    community: &ApubCommunity,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let object = FollowCommunity::new(actor, community, context)?;
    let undo = UndoFollowCommunity {
      actor: ObjectId::new(actor.actor_id()),
      object,
      kind: UndoType::Undo,
      id: generate_activity_id(
        UndoType::Undo,
        &context.settings().get_protocol_and_hostname(),
      )?,
      unparsed: Default::default(),
    };
    let inbox = vec![community.shared_inbox_or_inbox_url()];
    send_lemmy_activity(context, &undo, &undo.id, actor, inbox, true).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoFollowCommunity {
  type DataType = LemmyContext;
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(&self.id, self.actor.inner(), &context.settings())?;
    verify_urls_match(self.actor.inner(), self.object.actor.inner())?;
    verify_person(&self.actor, context, request_counter).await?;
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
      .object
      .dereference(context, request_counter)
      .await?;

    let community_follower_form = CommunityFollowerForm {
      community_id: community.id,
      person_id: person.id,
      pending: false,
    };

    // This will fail if they aren't a follower, but ignore the error.
    blocking(context.pool(), move |conn| {
      CommunityFollower::unfollow(conn, &community_follower_form).ok()
    })
    .await?;
    Ok(())
  }
}
