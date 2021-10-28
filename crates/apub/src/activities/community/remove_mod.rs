use crate::{
  activities::{
    community::{
      announce::{AnnouncableActivities, GetCommunity},
      get_community_from_moderators_url,
      send_to_community,
    },
    generate_activity_id,
    verify_activity,
    verify_add_remove_moderator_target,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
  },
  context::lemmy_context,
  fetcher::object_id::ObjectId,
  generate_moderators_url,
  objects::{community::ApubCommunity, person::ApubPerson},
};
use activitystreams::{
  activity::kind::RemoveType,
  base::AnyBase,
  primitives::OneOrMany,
  public,
  unparsed::Unparsed,
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  data::Data,
  traits::{ActivityFields, ActivityHandler, ActorType},
};
use lemmy_db_schema::{
  source::community::{CommunityModerator, CommunityModeratorForm},
  traits::Joinable,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMod {
  actor: ObjectId<ApubPerson>,
  to: Vec<Url>,
  pub(in crate::activities) object: ObjectId<ApubPerson>,
  cc: Vec<Url>,
  #[serde(rename = "type")]
  kind: RemoveType,
  pub(in crate::activities) target: Url,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl RemoveMod {
  pub async fn send(
    community: &ApubCommunity,
    removed_mod: &ApubPerson,
    actor: &ApubPerson,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let id = generate_activity_id(
      RemoveType::Remove,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let remove = RemoveMod {
      actor: ObjectId::new(actor.actor_id()),
      to: vec![public()],
      object: ObjectId::new(removed_mod.actor_id()),
      target: generate_moderators_url(&community.actor_id)?.into(),
      id: id.clone(),
      context: lemmy_context(),
      cc: vec![community.actor_id()],
      kind: RemoveType::Remove,
      unparsed: Default::default(),
    };

    let activity = AnnouncableActivities::RemoveMod(remove);
    let inboxes = vec![removed_mod.shared_inbox_or_inbox_url()];
    send_to_community(activity, &id, actor, community, inboxes, context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for RemoveMod {
  type DataType = LemmyContext;
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to)?;
    verify_activity(self, &context.settings())?;
    let community = self.get_community(context, request_counter).await?;
    verify_person_in_community(&self.actor, &community, context, request_counter).await?;
    verify_mod_action(&self.actor, &community, context, request_counter).await?;
    verify_add_remove_moderator_target(&self.target, &community)?;
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = self.get_community(context, request_counter).await?;
    let remove_mod = self.object.dereference(context, request_counter).await?;

    let form = CommunityModeratorForm {
      community_id: community.id,
      person_id: remove_mod.id,
    };
    blocking(context.pool(), move |conn| {
      CommunityModerator::leave(conn, &form)
    })
    .await??;
    // TODO: send websocket notification about removed mod
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl GetCommunity for RemoveMod {
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    get_community_from_moderators_url(&self.target, context, request_counter).await
  }
}
