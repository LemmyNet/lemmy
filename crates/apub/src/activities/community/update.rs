use crate::{
  activities::{community::send_activity_in_community, generate_activity_id, verify_mod_action},
  activity_lists::AnnouncableActivities,
  insert_received_activity,
  protocol::activities::community::update::UpdateCommunity,
};
use activitypub_federation::{
  config::Data,
  kinds::activity::UpdateType,
  traits::{ActivityHandler, Actor, Object},
};
use lemmy_api_common::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, person::ApubPerson},
  utils::{
    functions::{generate_to, verify_person_in_community, verify_visibility},
    protocol::InCommunity,
  },
};
use lemmy_db_schema::{
  source::{
    activity::ActivitySendTargets,
    community::Community,
    mod_log::moderator::{ModChangeCommunityVisibility, ModChangeCommunityVisibilityForm},
    person::Person,
  },
  traits::Crud,
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
    to: generate_to(&community)?,
    object: Box::new(community.clone().into_json(&context).await?),
    cc: vec![community.id()],
    kind: UpdateType::Update,
    id: id.clone(),
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

  async fn verify(&self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let community = self.community(context).await?;
    verify_visibility(&self.to, &self.cc, &community)?;
    verify_person_in_community(&self.actor, &community, context).await?;
    verify_mod_action(&self.actor, &community, context).await?;
    ApubCommunity::verify(&self.object, &community.ap_id.clone().into(), context).await?;
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    insert_received_activity(&self.id, context).await?;
    let old_community = self.community(context).await?;

    let community = ApubCommunity::from_json(*self.object, context).await?;

    if old_community.visibility != community.visibility {
      let actor = self.actor.dereference(context).await?;
      let form = ModChangeCommunityVisibilityForm {
        mod_person_id: actor.id,
        community_id: old_community.id,
        visibility: old_community.visibility,
      };
      ModChangeCommunityVisibility::create(&mut context.pool(), &form).await?;
    }
    Ok(())
  }
}
