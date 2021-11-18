use crate::{
  activities::{
    community::{announce::GetCommunity, send_activity_in_community},
    generate_activity_id,
    verify_activity,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::community::block_user::BlockUserFromCommunity,
};
use activitystreams::{activity::kind::BlockType, public};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  traits::{ActivityHandler, ActorType},
};
use lemmy_db_schema::{
  source::community::{
    CommunityFollower,
    CommunityFollowerForm,
    CommunityPersonBan,
    CommunityPersonBanForm,
  },
  traits::{Bannable, Followable},
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;

impl BlockUserFromCommunity {
  pub(in crate::activities::community) fn new(
    community: &ApubCommunity,
    target: &ApubPerson,
    actor: &ApubPerson,
    context: &LemmyContext,
  ) -> Result<BlockUserFromCommunity, LemmyError> {
    Ok(BlockUserFromCommunity {
      actor: ObjectId::new(actor.actor_id()),
      to: Some(public().into()),
      object: ObjectId::new(target.actor_id()),
      cc: Some(community.actor_id().into()),
      target: ObjectId::new(community.actor_id()),
      kind: BlockType::Block,
      id: generate_activity_id(
        BlockType::Block,
        &context.settings().get_protocol_and_hostname(),
      )?,
      unparsed: Default::default(),
    })
  }

  pub async fn send(
    community: &ApubCommunity,
    target: &ApubPerson,
    actor: &ApubPerson,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let block = BlockUserFromCommunity::new(community, target, actor, context)?;
    let block_id = block.id.clone();

    let activity = AnnouncableActivities::BlockUserFromCommunity(block);
    let inboxes = vec![target.shared_inbox_or_inbox_url()];
    send_activity_in_community(activity, &block_id, actor, community, inboxes, context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for BlockUserFromCommunity {
  type DataType = LemmyContext;
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to, &self.cc)?;
    verify_activity(&self.id, self.actor.inner(), &context.settings())?;
    let community = self.get_community(context, request_counter).await?;
    verify_person_in_community(&self.actor, &community, context, request_counter).await?;
    verify_mod_action(&self.actor, &community, context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = self.get_community(context, request_counter).await?;
    let blocked_user = self.object.dereference(context, request_counter).await?;

    let community_user_ban_form = CommunityPersonBanForm {
      community_id: community.id,
      person_id: blocked_user.id,
    };

    blocking(context.pool(), move |conn: &'_ _| {
      CommunityPersonBan::ban(conn, &community_user_ban_form)
    })
    .await??;

    // Also unsubscribe them from the community, if they are subscribed
    let community_follower_form = CommunityFollowerForm {
      community_id: community.id,
      person_id: blocked_user.id,
      pending: false,
    };
    blocking(context.pool(), move |conn: &'_ _| {
      CommunityFollower::unfollow(conn, &community_follower_form)
    })
    .await?
    .ok();

    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl GetCommunity for BlockUserFromCommunity {
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    self.target.dereference(context, request_counter).await
  }
}
