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
  objects::{community::ApubCommunity, person::ApubPerson, read_from_string_or_source_opt},
  protocol::{activities::community::update::UpdateCommunity, InCommunity},
};
use activitypub_federation::{
  config::Data,
  kinds::{activity::UpdateType, public},
  traits::{ActivityHandler, Actor, Object},
};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{
    activity::ActivitySendTargets,
    community::{Community, CommunityUpdateForm},
    person::Person,
  },
  traits::Crud,
  utils::naive_now,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

pub(crate) async fn send_update_community(
  community: Community,
  actor: Person,
  context: Data<LemmyContext>,
) -> LemmyResult<()> {
  let community: ApubCommunity = community.into();
  let actor: ApubPerson = actor.into();
  let id = generate_activity_id(
    UpdateType::Update,
    &context.settings().get_protocol_and_hostname(),
  )?;
  let update = UpdateCommunity {
    actor: actor.id().into(),
    to: vec![public()],
    object: Box::new(community.clone().into_json(&context).await?),
    cc: vec![community.id()],
    kind: UpdateType::Update,
    id: id.clone(),
    audience: Some(community.id().into()),
  };

  let activity = AnnouncableActivities::UpdateCommunity(update);
  send_activity_in_community(
    activity,
    &actor,
    &community,
    ActivitySendTargets::empty(),
    true,
    &context,
  )
  .await
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
  async fn verify(&self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    verify_is_public(&self.to, &self.cc)?;
    let community = self.community(context).await?;
    verify_person_in_community(&self.actor, &community, context).await?;
    verify_mod_action(&self.actor, &community, context).await?;
    ApubCommunity::verify(&self.object, &community.actor_id.clone().into(), context).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    insert_received_activity(&self.id, context).await?;
    let community = self.community(context).await?;

    let community_update_form = CommunityUpdateForm {
      title: Some(self.object.name.unwrap_or(self.object.preferred_username)),
      description: Some(read_from_string_or_source_opt(
        &self.object.summary,
        &None,
        &self.object.source,
      )),
      published: self.object.published.map(Into::into),
      updated: Some(self.object.updated.map(Into::into)),
      nsfw: Some(self.object.sensitive.unwrap_or(false)),
      actor_id: Some(self.object.id.into()),
      public_key: Some(self.object.public_key.public_key_pem),
      last_refreshed_at: Some(naive_now()),
      icon: Some(self.object.icon.map(|i| i.url.into())),
      banner: Some(self.object.image.map(|i| i.url.into())),
      followers_url: self.object.followers.map(Into::into),
      inbox_url: Some(self.object.inbox.into()),
      shared_inbox_url: Some(self.object.endpoints.map(|e| e.shared_inbox.into())),
      moderators_url: self.object.attributed_to.map(Into::into),
      posting_restricted_to_mods: self.object.posting_restricted_to_mods,
      featured_url: self.object.featured.map(Into::into),
      ..Default::default()
    };

    Community::update(&mut context.pool(), community.id, &community_update_form).await?;
    Ok(())
  }
}
