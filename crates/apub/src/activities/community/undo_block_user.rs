use crate::{
  activities::{
    community::{
      announce::AnnouncableActivities,
      block_user::BlockUserFromCommunity,
      send_to_community,
    },
    generate_activity_id,
    verify_activity,
    verify_mod_action,
    verify_person_in_community,
  },
  context::lemmy_context,
  fetcher::object_id::ObjectId,
  objects::{community::ApubCommunity, person::ApubPerson},
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
  values::PublicUrl,
};
use lemmy_db_schema::{
  source::community::{CommunityPersonBan, CommunityPersonBanForm},
  traits::Bannable,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct UndoBlockUserFromCommunity {
  actor: ObjectId<ApubPerson>,
  to: [PublicUrl; 1],
  object: BlockUserFromCommunity,
  cc: [ObjectId<ApubCommunity>; 1],
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
    community: &ApubCommunity,
    target: &ApubPerson,
    actor: &ApubPerson,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let block = BlockUserFromCommunity::new(community, target, actor, context)?;

    let id = generate_activity_id(
      UndoType::Undo,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let undo = UndoBlockUserFromCommunity {
      actor: ObjectId::new(actor.actor_id()),
      to: [PublicUrl::Public],
      object: block,
      cc: [ObjectId::new(community.actor_id())],
      kind: UndoType::Undo,
      id: id.clone(),
      context: lemmy_context(),
      unparsed: Default::default(),
    };

    let activity = AnnouncableActivities::UndoBlockUserFromCommunity(undo);
    let inboxes = vec![target.shared_inbox_or_inbox_url()];
    send_to_community(activity, &id, actor, community, inboxes, context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoBlockUserFromCommunity {
  type DataType = LemmyContext;
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self, &context.settings())?;
    verify_person_in_community(&self.actor, &self.cc[0], context, request_counter).await?;
    verify_mod_action(&self.actor, &self.cc[0], context, request_counter).await?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = self.cc[0].dereference(context, request_counter).await?;
    let blocked_user = self
      .object
      .object
      .dereference(context, request_counter)
      .await?;

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
