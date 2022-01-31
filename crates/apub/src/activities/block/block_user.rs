use crate::{
  activities::{
    block::{generate_cc, generate_instance_inboxes, SiteOrCommunity},
    community::{announce::GetCommunity, send_activity_in_community},
    generate_activity_id,
    send_lemmy_activity,
    verify_activity,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::block::block_user::BlockUser,
};
use activitystreams_kinds::{activity::BlockType, public};
use anyhow::anyhow;
use chrono::NaiveDateTime;
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
use lemmy_utils::{utils::convert_datetime, LemmyError};
use lemmy_websocket::LemmyContext;

impl BlockUser {
  pub(in crate::activities::block) async fn new(
    target: &SiteOrCommunity,
    user: &ApubPerson,
    mod_: &ApubPerson,
    remove_data: Option<bool>,
    expires: Option<NaiveDateTime>,
    context: &LemmyContext,
  ) -> Result<BlockUser, LemmyError> {
    Ok(BlockUser {
      actor: ObjectId::new(mod_.actor_id()),
      to: vec![public()],
      object: ObjectId::new(user.actor_id()),
      cc: generate_cc(target, context.pool()).await?,
      target: target.id(),
      kind: BlockType::Block,
      remove_data,
      id: generate_activity_id(
        BlockType::Block,
        &context.settings().get_protocol_and_hostname(),
      )?,
      expires: expires.map(convert_datetime),
      unparsed: Default::default(),
    })
  }

  #[tracing::instrument(skip_all)]
  pub async fn send(
    target: &SiteOrCommunity,
    user: &ApubPerson,
    mod_: &ApubPerson,
    remove_data: bool,
    expires: Option<NaiveDateTime>,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let block = BlockUser::new(target, user, mod_, Some(remove_data), expires, context).await?;
    let block_id = block.id.clone();

    match target {
      SiteOrCommunity::Site(_) => {
        let inboxes = generate_instance_inboxes(user, context.pool()).await?;
        send_lemmy_activity(context, &block, &block_id, mod_, inboxes, false).await
      }
      SiteOrCommunity::Community(c) => {
        let activity = AnnouncableActivities::BlockUser(block);
        let inboxes = vec![user.shared_inbox_or_inbox_url()];
        send_activity_in_community(activity, &block_id, mod_, c, inboxes, context).await
      }
    }
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for BlockUser {
  type DataType = LemmyContext;

  #[tracing::instrument(skip_all)]
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

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = self.get_community(context, request_counter).await?;
    let blocked_user = self
      .object
      .dereference(context, context.client(), request_counter)
      .await?;

    let community_user_ban_form = CommunityPersonBanForm {
      community_id: community.id,
      person_id: blocked_user.id,
      expires: Some(self.expires.map(|u| u.naive_local())),
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
impl GetCommunity for BlockUser {
  #[tracing::instrument(skip_all)]
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let target = self
      .target
      .dereference(context, context.client(), request_counter)
      .await?;
    match target {
      SiteOrCommunity::Community(c) => Ok(c),
      SiteOrCommunity::Site(_) => Err(anyhow!("Calling get_community() on site activity").into()),
    }
  }
}
