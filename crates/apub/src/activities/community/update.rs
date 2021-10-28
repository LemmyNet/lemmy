use crate::{
  activities::{
    community::{
      announce::{AnnouncableActivities, GetCommunity},
      send_to_community,
    },
    generate_activity_id,
    verify_activity,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
  },
  context::lemmy_context,
  fetcher::object_id::ObjectId,
  objects::{
    community::{ApubCommunity, Group},
    person::ApubPerson,
  },
};
use activitystreams::{
  activity::kind::UpdateType,
  base::AnyBase,
  primitives::OneOrMany,
  public,
  unparsed::Unparsed,
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  data::Data,
  traits::{ActivityFields, ActivityHandler, ActorType, ApubObject},
};
use lemmy_db_schema::{
  source::community::{Community, CommunityForm},
  traits::Crud,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::{send::send_community_ws_message, LemmyContext, UserOperationCrud};
use serde::{Deserialize, Serialize};
use url::Url;

/// This activity is received from a remote community mod, and updates the description or other
/// fields of a local community.
#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCommunity {
  actor: ObjectId<ApubPerson>,
  to: Vec<Url>,
  // TODO: would be nice to use a separate struct here, which only contains the fields updated here
  object: Group,
  cc: Vec<Url>,
  #[serde(rename = "type")]
  kind: UpdateType,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl UpdateCommunity {
  pub async fn send(
    community: &ApubCommunity,
    actor: &ApubPerson,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let id = generate_activity_id(
      UpdateType::Update,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let update = UpdateCommunity {
      actor: ObjectId::new(actor.actor_id()),
      to: vec![public()],
      object: community.to_apub(context).await?,
      cc: vec![community.actor_id()],
      kind: UpdateType::Update,
      id: id.clone(),
      context: lemmy_context(),
      unparsed: Default::default(),
    };

    let activity = AnnouncableActivities::UpdateCommunity(Box::new(update));
    send_to_community(activity, &id, actor, community, vec![], context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UpdateCommunity {
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
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = self.get_community(context, request_counter).await?;

    let updated_community = Group::from_apub_to_form(
      &self.object,
      &community.actor_id.clone().into(),
      &context.settings(),
    )
    .await?;
    let cf = CommunityForm {
      name: updated_community.name,
      title: updated_community.title,
      description: updated_community.description,
      nsfw: updated_community.nsfw,
      // TODO: icon and banner would be hosted on the other instance, ideally we would copy it to ours
      icon: updated_community.icon,
      banner: updated_community.banner,
      ..CommunityForm::default()
    };
    let updated_community = blocking(context.pool(), move |conn| {
      Community::update(conn, community.id, &cf)
    })
    .await??;

    send_community_ws_message(
      updated_community.id,
      UserOperationCrud::EditCommunity,
      None,
      None,
      context,
    )
    .await?;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl GetCommunity for UpdateCommunity {
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let cid = ObjectId::new(self.object.id.clone());
    cid.dereference(context, request_counter).await
  }
}
