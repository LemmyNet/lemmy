use crate::{
  activities::{
    community::{announce::AnnouncableActivities, send_to_community},
    generate_activity_id,
    verify_activity,
    verify_add_remove_moderator_target,
    verify_mod_action,
    verify_person_in_community,
  },
  context::lemmy_context,
  fetcher::object_id::ObjectId,
  generate_moderators_url,
  migrations::PublicUrlMigration,
  objects::{community::ApubCommunity, person::ApubPerson},
};
use activitystreams::{
  activity::kind::RemoveType,
  base::AnyBase,
  primitives::OneOrMany,
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
  to: PublicUrlMigration,
  pub(in crate::activities) object: ObjectId<ApubPerson>,
  cc: [ObjectId<ApubCommunity>; 1],
  #[serde(rename = "type")]
  kind: RemoveType,
  // if target is set, this is means remove mod from community
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
      to: PublicUrlMigration::create(),
      object: ObjectId::new(removed_mod.actor_id()),
      target: generate_moderators_url(&community.actor_id)?.into(),
      id: id.clone(),
      context: lemmy_context(),
      cc: [ObjectId::new(community.actor_id())],
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
    verify_activity(self, &context.settings())?;
    verify_person_in_community(&self.actor, &self.cc[0], context, request_counter).await?;
    verify_mod_action(&self.actor, &self.cc[0], context, request_counter).await?;
    verify_add_remove_moderator_target(&self.target, &self.cc[0])?;
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = self.cc[0].dereference(context, request_counter).await?;
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
