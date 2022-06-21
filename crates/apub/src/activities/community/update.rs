use crate::{
  activities::{
    community::{announce::GetCommunity, send_activity_in_community},
    generate_activity_id,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::community::update::UpdateCommunity,
  ActorType,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  traits::{ActivityHandler, ApubObject},
};
use activitystreams_kinds::{activity::UpdateType, public};
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{
  source::community::{Community, CommunityForm},
  traits::Crud,
};
use lemmy_utils::error::LemmyError;
use lemmy_websocket::{send::send_community_ws_message, LemmyContext, UserOperationCrud};
use url::Url;

impl UpdateCommunity {
  #[tracing::instrument(skip_all)]
  pub async fn send(
    community: ApubCommunity,
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
      object: Box::new(community.clone().into_apub(context).await?),
      cc: vec![community.actor_id()],
      kind: UpdateType::Update,
      id: id.clone(),
      unparsed: Default::default(),
    };

    let activity = AnnouncableActivities::UpdateCommunity(update);
    send_activity_in_community(activity, actor, &community, vec![], context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UpdateCommunity {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to, &self.cc)?;
    let community = self.get_community(context, request_counter).await?;
    verify_person_in_community(&self.actor, &community, context, request_counter).await?;
    verify_mod_action(
      &self.actor,
      self.object.id.inner(),
      &community,
      context,
      request_counter,
    )
    .await?;
    ApubCommunity::verify(
      &self.object,
      &community.actor_id.clone().into(),
      context,
      request_counter,
    )
    .await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community = self.get_community(context, request_counter).await?;

    let updated_community = self.object.into_form();
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
  #[tracing::instrument(skip_all)]
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let cid = ObjectId::new(self.object.id.clone());
    cid
      .dereference(context, local_instance(context), request_counter)
      .await
  }
}
