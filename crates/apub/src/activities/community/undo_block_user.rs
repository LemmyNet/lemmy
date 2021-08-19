use crate::{
  activities::{
    community::{announce::AnnouncableActivities, block_user::BlockUserFromCommunity},
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
use activitystreams::{
  activity::kind::UndoType,
  base::AnyBase,
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{values::PublicUrl, ActivityFields, ActivityHandler};
use lemmy_db_queries::Bannable;
use lemmy_db_schema::source::{
  community::{Community, CommunityPersonBan, CommunityPersonBanForm},
  person::Person,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct UndoBlockUserFromCommunity {
  actor: Url,
  to: PublicUrl,
  object: BlockUserFromCommunity,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: UndoType,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl UndoBlockUserFromCommunity {
  pub async fn send(
    community: &Community,
    target: &Person,
    actor: &Person,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let block = BlockUserFromCommunity::new(community, target, actor)?;

    let id = generate_activity_id(UndoType::Undo)?;
    let undo = UndoBlockUserFromCommunity {
      actor: actor.actor_id(),
      to: PublicUrl::Public,
      object: block,
      cc: [community.actor_id()],
      kind: UndoType::Undo,
      id: id.clone(),
      context: lemmy_context(),
      unparsed: Default::default(),
    };

    let activity = AnnouncableActivities::UndoBlockUserFromCommunity(undo);
    let inboxes = vec![target.get_shared_inbox_or_inbox_url()];
    send_to_community_new(activity, &id, actor, community, inboxes, context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoBlockUserFromCommunity {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self)?;
    verify_person_in_community(&self.actor, &self.cc[0], context, request_counter).await?;
    verify_mod_action(&self.actor, self.cc[0].clone(), context).await?;
    self.object.verify(context, request_counter).await?;
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
      get_or_fetch_and_upsert_person(&self.object.object, context, request_counter).await?;

    let community_user_ban_form = CommunityPersonBanForm {
      community_id: community.id,
      person_id: blocked_user.id,
    };

    blocking(context.pool(), move |conn: &'_ _| {
      CommunityPersonBan::unban(conn, &community_user_ban_form)
    })
    .await??;

    Ok(())
  }
}
