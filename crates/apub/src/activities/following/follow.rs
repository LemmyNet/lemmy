use crate::{
  activities::{
    following::accept::AcceptFollowCommunity,
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
  activity::kind::FollowType,
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
pub struct FollowCommunity {
  actor: Url,
  pub(in crate::activities::following) to: Url,
  pub(in crate::activities::following) object: Url,
  #[serde(rename = "type")]
  kind: FollowType,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl FollowCommunity {
  pub(in crate::activities::following) fn new(
    actor: &Person,
    community: &Community,
  ) -> Result<FollowCommunity, LemmyError> {
    Ok(FollowCommunity {
      actor: actor.actor_id(),
      to: community.actor_id(),
      object: community.actor_id(),
      kind: FollowType::Follow,
      id: generate_activity_id(FollowType::Follow)?,
      context: lemmy_context(),
      unparsed: Default::default(),
    })
  }
  pub async fn send(
    actor: &Person,
    community: &Community,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let community_follower_form = CommunityFollowerForm {
      community_id: community.id,
      person_id: actor.id,
      pending: true,
    };
    blocking(context.pool(), move |conn| {
      CommunityFollower::follow(conn, &community_follower_form).ok()
    })
    .await?;

    let follow = FollowCommunity::new(actor, community)?;
    let inbox = vec![community.inbox_url.clone().into()];
    send_activity_new(context, &follow, &follow.id, actor, inbox, true).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for FollowCommunity {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self)?;
    verify_urls_match(&self.to, &self.object)?;
    verify_person(&self.actor, context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor = get_or_fetch_and_upsert_person(&self.actor, context, request_counter).await?;
    let community =
      get_or_fetch_and_upsert_community(&self.object, context, request_counter).await?;
    let community_follower_form = CommunityFollowerForm {
      community_id: community.id,
      person_id: actor.id,
      pending: false,
    };

    // This will fail if they're already a follower, but ignore the error.
    blocking(context.pool(), move |conn| {
      CommunityFollower::follow(conn, &community_follower_form).ok()
    })
    .await?;

    AcceptFollowCommunity::send(self, context).await
  }
}
