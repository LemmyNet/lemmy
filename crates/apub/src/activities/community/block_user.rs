use crate::{
  activities::{
    community::announce::AnnouncableActivities,
    generate_activity_id,
    verify_activity,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_queue::send_to_community_new,
  extensions::context::lemmy_context,
  fetcher::object_id::ObjectId,
  ActorType,
};
use activitystreams::{
  activity::kind::BlockType,
  base::AnyBase,
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  data::Data,
  traits::{ActivityFields, ActivityHandler},
  values::PublicUrl,
};
use lemmy_db_queries::{Bannable, Followable};
use lemmy_db_schema::source::{
  community::{
    Community,
    CommunityFollower,
    CommunityFollowerForm,
    CommunityPersonBan,
    CommunityPersonBanForm,
  },
  person::Person,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct BlockUserFromCommunity {
  actor: ObjectId<Person>,
  to: [PublicUrl; 1],
  pub(in crate::activities::community) object: ObjectId<Person>,
  cc: [ObjectId<Community>; 1],
  #[serde(rename = "type")]
  kind: BlockType,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl BlockUserFromCommunity {
  pub(in crate::activities::community) fn new(
    community: &Community,
    target: &Person,
    actor: &Person,
    context: &LemmyContext,
  ) -> Result<BlockUserFromCommunity, LemmyError> {
    Ok(BlockUserFromCommunity {
      actor: ObjectId::new(actor.actor_id()),
      to: [PublicUrl::Public],
      object: ObjectId::new(target.actor_id()),
      cc: [ObjectId::new(community.actor_id())],
      kind: BlockType::Block,
      id: generate_activity_id(
        BlockType::Block,
        &context.settings().get_protocol_and_hostname(),
      )?,
      context: lemmy_context(),
      unparsed: Default::default(),
    })
  }

  pub async fn send(
    community: &Community,
    target: &Person,
    actor: &Person,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let block = BlockUserFromCommunity::new(community, target, actor, context)?;
    let block_id = block.id.clone();

    let activity = AnnouncableActivities::BlockUserFromCommunity(block);
    let inboxes = vec![target.get_shared_inbox_or_inbox_url()];
    send_to_community_new(activity, &block_id, actor, community, inboxes, context).await
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
    verify_activity(self, &context.settings())?;
    verify_person_in_community(&self.actor, &self.cc[0], context, request_counter).await?;
    verify_mod_action(&self.actor, self.cc[0].clone(), context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = self.cc[0].dereference(context, request_counter).await?;
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
