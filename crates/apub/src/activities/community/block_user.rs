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
  fetcher::{community::get_or_fetch_and_upsert_community, person::get_or_fetch_and_upsert_person},
  ActorType,
};
use activitystreams::activity::kind::BlockType;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{values::PublicUrl, ActivityCommonFields, ActivityHandler};
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
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockUserFromCommunity {
  to: PublicUrl,
  pub(in crate::activities::community) object: Url,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: BlockType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

impl BlockUserFromCommunity {
  pub(in crate::activities::community) fn new(
    id: Url,
    community: &Community,
    target: &Person,
    actor: &Person,
  ) -> BlockUserFromCommunity {
    BlockUserFromCommunity {
      to: PublicUrl::Public,
      object: target.actor_id(),
      cc: [community.actor_id()],
      kind: BlockType::Block,
      common: ActivityCommonFields {
        context: lemmy_context(),
        id,
        actor: actor.actor_id(),
        unparsed: Default::default(),
      },
    }
  }

  pub async fn send(
    community: &Community,
    target: &Person,
    actor: &Person,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let id = generate_activity_id(BlockType::Block)?;
    let block = BlockUserFromCommunity::new(id.clone(), community, target, actor);

    let activity = AnnouncableActivities::BlockUserFromCommunity(block);
    let inboxes = vec![target.get_shared_inbox_or_inbox_url()];
    send_to_community_new(activity, &id, actor, community, inboxes, context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for BlockUserFromCommunity {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    verify_person_in_community(&self.common.actor, &self.cc[0], context, request_counter).await?;
    verify_mod_action(&self.common.actor, self.cc[0].clone(), context).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community =
      get_or_fetch_and_upsert_community(&self.cc[0], context, request_counter).await?;
    let blocked_user =
      get_or_fetch_and_upsert_person(&self.object, context, request_counter).await?;

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

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
