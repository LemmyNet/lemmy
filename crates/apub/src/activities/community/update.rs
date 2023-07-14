use crate::{
  activities::{
    community::send_activity_in_community,
    generate_activity_id,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  insert_received_activity,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{activities::community::update::UpdateCommunity, InCommunity},
  SendActivity,
};
use activitypub_federation::{
  config::Data,
  kinds::{activity::UpdateType, public},
  traits::{ActivityHandler, Actor, Object},
};
use lemmy_api_common::{
  community::{CommunityResponse, EditCommunity, HideCommunity},
  context::LemmyContext,
  utils::local_user_view_from_jwt,
};
use lemmy_db_schema::{source::community::Community, traits::Crud};
use lemmy_utils::error::LemmyError;
use url::Url;

#[async_trait::async_trait]
impl SendActivity for EditCommunity {
  type Response = CommunityResponse;

  async fn send_activity(
    request: &Self,
    _response: &Self::Response,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let local_user_view = local_user_view_from_jwt(&request.auth, context).await?;
    let community = Community::read(&mut context.pool(), request.community_id).await?;
    UpdateCommunity::send(community.into(), &local_user_view.person.into(), context).await
  }
}

impl UpdateCommunity {
  #[tracing::instrument(skip_all)]
  pub async fn send(
    community: ApubCommunity,
    actor: &ApubPerson,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let id = generate_activity_id(
      UpdateType::Update,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let update = UpdateCommunity {
      actor: actor.id().into(),
      to: vec![public()],
      object: Box::new(community.clone().into_json(context).await?),
      cc: vec![community.id()],
      kind: UpdateType::Update,
      id: id.clone(),
      audience: Some(community.id().into()),
    };

    let activity = AnnouncableActivities::UpdateCommunity(update);
    send_activity_in_community(activity, actor, &community, vec![], true, context).await
  }
}

#[async_trait::async_trait]
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
  async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), LemmyError> {
    insert_received_activity(&self.id, context).await?;
    verify_is_public(&self.to, &self.cc)?;
    let community = self.community(context).await?;
    verify_person_in_community(&self.actor, &community, context).await?;
    verify_mod_action(&self.actor, self.object.id.inner(), community.id, context).await?;
    ApubCommunity::verify(&self.object, &community.actor_id.clone().into(), context).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<Self::DataType>) -> Result<(), LemmyError> {
    let community = self.community(context).await?;

    let community_update_form = self.object.into_update_form();

    Community::update(&mut context.pool(), community.id, &community_update_form).await?;
    Ok(())
  }
}

#[async_trait::async_trait]
impl SendActivity for HideCommunity {
  type Response = CommunityResponse;

  async fn send_activity(
    request: &Self,
    _response: &Self::Response,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let local_user_view = local_user_view_from_jwt(&request.auth, context).await?;
    let community = Community::read(&mut context.pool(), request.community_id).await?;
    UpdateCommunity::send(community.into(), &local_user_view.person.into(), context).await
  }
}
