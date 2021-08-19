use crate::{
  activities::{
    following::follow::FollowCommunity,
    generate_activity_id,
    verify_activity,
    verify_person,
  },
  activity_queue::send_activity_new,
  extensions::context::lemmy_context,
  fetcher::{community::get_or_fetch_and_upsert_community, person::get_or_fetch_and_upsert_person},
  ActorType,
};
use activitystreams::{
  activity::kind::UndoType,
  base::AnyBase,
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{verify_urls_match, ActivityFields, ActivityHandler};
use lemmy_db_queries::Followable;
use lemmy_db_schema::source::{
  community::{Community, CommunityFollower, CommunityFollowerForm},
  person::Person,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct UndoFollowCommunity {
  actor: Url,
  to: Url,
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
    actor: &Person,
    community: &Community,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let object = FollowCommunity::new(actor, community)?;
    let undo = UndoFollowCommunity {
      actor: actor.actor_id(),
      to: community.actor_id(),
      object,
      kind: UndoType::Undo,
      id: generate_activity_id(UndoType::Undo)?,
      context: lemmy_context(),
      unparsed: Default::default(),
    };
    let inbox = vec![community.get_shared_inbox_or_inbox_url()];
    send_activity_new(context, &undo, &undo.id, actor, inbox, true).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoFollowCommunity {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self)?;
    verify_urls_match(&self.to, &self.object.object)?;
    verify_urls_match(&self.actor, self.object.actor())?;
    verify_person(&self.actor, context, request_counter).await?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor = get_or_fetch_and_upsert_person(&self.actor, context, request_counter).await?;
    let community = get_or_fetch_and_upsert_community(&self.to, context, request_counter).await?;

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
