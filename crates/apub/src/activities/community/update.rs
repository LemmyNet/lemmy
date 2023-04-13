use crate::{
  activities::{
    community::send_activity_in_community,
    generate_activity_id,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{activities::community::update::UpdateCommunity, InCommunity},
  ActorType,
  SendActivity,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  traits::{ActivityHandler, ApubObject},
};
use activitystreams_kinds::{activity::UpdateType, public};
use lemmy_api_common::{
  community::{CommunityResponse, EditCommunity, HideCommunity},
  context::LemmyContext,
  utils::get_local_user_view_from_jwt,
  websocket::UserOperationCrud,
};
use lemmy_db_schema::{source::community::Community, traits::Crud};
use lemmy_utils::error::LemmyError;
use url::Url;

#[async_trait::async_trait(?Send)]
impl SendActivity for EditCommunity {
  type Response = CommunityResponse;

  async fn send_activity(
    request: &Self,
    _response: &Self::Response,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let local_user_view =
      get_local_user_view_from_jwt(&request.auth, context.pool(), context.secret()).await?;
    let community = Community::read(context.pool(), request.community_id).await?;
    UpdateCommunity::send(community.into(), &local_user_view.person.into(), context).await
  }
}

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
      audience: Some(ObjectId::new(community.actor_id())),
    };

    let activity = AnnouncableActivities::UpdateCommunity(update);
    send_activity_in_community(activity, actor, &community, vec![], true, context).await
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
    let community = self.community(context, request_counter).await?;
    verify_person_in_community(&self.actor, &community, context, request_counter).await?;
    verify_mod_action(
      &self.actor,
      self.object.id.inner(),
      community.id,
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
    let community = self.community(context, request_counter).await?;

    let community_update_form = self.object.into_update_form();

    let updated_community =
      Community::update(context.pool(), community.id, &community_update_form).await?;

    context
      .send_community_ws_message(
        &UserOperationCrud::EditCommunity,
        updated_community.id,
        None,
        None,
      )
      .await?;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl SendActivity for HideCommunity {
  type Response = CommunityResponse;

  async fn send_activity(
    request: &Self,
    _response: &Self::Response,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let local_user_view =
      get_local_user_view_from_jwt(&request.auth, context.pool(), context.secret()).await?;
    let community = Community::read(context.pool(), request.community_id).await?;
    UpdateCommunity::send(community.into(), &local_user_view.person.into(), context).await
  }
}
