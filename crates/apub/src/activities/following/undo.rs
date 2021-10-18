use crate::{
  activities::{
    following::follow::FollowCommunity,
    generate_activity_id,
    verify_activity,
    verify_person,
  },
  context::lemmy_context,
  fetcher::object_id::ObjectId,
  objects::{community::ApubCommunity, person::ApubPerson},
  send_lemmy_activity,
};
use activitystreams::{
  activity::kind::UndoType,
  base::AnyBase,
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  data::Data,
  traits::{ActivityFields, ActivityHandler, ActorType},
  verify::verify_urls_match,
};
use lemmy_db_schema::{
  source::community::{CommunityFollower, CommunityFollowerForm},
  traits::Followable,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct UndoFollowCommunity {
  actor: ObjectId<ApubPerson>,
  to: ObjectId<ApubCommunity>,
  object: FollowCommunity,
  #[serde(rename = "type")]
  kind: UndoType,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl UndoFollowCommunity {
  pub async fn send(
    actor: &ApubPerson,
    community: &ApubCommunity,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let object = FollowCommunity::new(actor, community, context)?;
    let undo = UndoFollowCommunity {
      actor: ObjectId::new(actor.actor_id()),
      to: ObjectId::new(community.actor_id()),
      object,
      kind: UndoType::Undo,
      id: generate_activity_id(
        UndoType::Undo,
        &context.settings().get_protocol_and_hostname(),
      )?,
      context: lemmy_context(),
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
    verify_activity(self, &context.settings())?;
    verify_urls_match(self.to.inner(), self.object.object.inner())?;
    verify_urls_match(self.actor(), self.object.actor())?;
    verify_person(&self.actor, context, request_counter).await?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor = self.actor.dereference(context, request_counter).await?;
    let community = self.to.dereference(context, request_counter).await?;

    let community_follower_form = CommunityFollowerForm {
      community_id: community.id,
      person_id: actor.id,
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
