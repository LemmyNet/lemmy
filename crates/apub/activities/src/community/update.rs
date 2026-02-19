use crate::{
  community::{AnnouncableActivities, send_activity_in_community},
  generate_activity_id,
  protocol::community::update::Update,
  send_lemmy_activity,
};
use activitypub_federation::{
  config::Data,
  kinds::{activity::UpdateType, public},
  traits::{Activity, Object},
};
use either::Either;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, multi_community::ApubMultiCommunity, person::ApubPerson},
  utils::{
    functions::{generate_to, verify_mod_action, verify_person_in_community, verify_visibility},
    protocol::InCommunity,
  },
};
use lemmy_db_schema::source::{
  activity::ActivitySendTargets,
  community::Community,
  modlog::{Modlog, ModlogInsertForm},
  multi_community::MultiCommunity,
  person::Person,
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
  let id = generate_activity_id(UpdateType::Update, &context)?;
  let update = Update {
    actor: actor.id().clone().into(),
    to: generate_to(&community)?,
    object: Either::Left(community.clone().into_json(&context).await?),
    cc: vec![community.id().clone()],
    kind: UpdateType::Update,
    id: id.clone(),
    audience: Some(community.ap_id.clone().into()),
  };

  let activity = AnnouncableActivities::UpdateCommunity(Box::new(update));
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

pub(crate) async fn send_update_multi_community(
  multi: MultiCommunity,
  actor: Person,
  context: Data<LemmyContext>,
) -> LemmyResult<()> {
  let multi: ApubMultiCommunity = multi.into();
  let actor: ApubPerson = actor.into();
  let id = generate_activity_id(UpdateType::Update, &context)?;
  let update = Update {
    actor: actor.id().clone().into(),
    to: vec![multi.ap_id.clone().into(), public()],
    object: Either::Right(multi.clone().into_json(&context).await?),
    cc: vec![],
    kind: UpdateType::Update,
    id: id.clone(),
    audience: Some(multi.ap_id.clone().into()),
  };

  let activity = AnnouncableActivities::UpdateCommunity(Box::new(update));
  let mut inboxes = ActivitySendTargets::empty();
  inboxes.add_inboxes(MultiCommunity::follower_inboxes(&mut context.pool(), multi.id).await?);
  send_lemmy_activity(&context, activity, &actor, inboxes, false).await
}

#[async_trait::async_trait]
impl Activity for Update {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    match &self.object {
      Either::Left(c) => {
        let community = self.community(context).await?;
        verify_visibility(&self.to, &self.cc, &community)?;
        verify_person_in_community(&self.actor, &community, context).await?;
        verify_mod_action(&self.actor, &community, context).await?;
        ApubCommunity::verify(c, &community.ap_id.clone().into(), context).await?;
      }
      Either::Right(m) => ApubMultiCommunity::verify(m, &self.id, context).await?,
    }
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    match &self.object {
      Either::Left(c) => {
        let old_community = self.community(context).await?;

        let community = ApubCommunity::from_json(c.clone(), context).await?;

        if old_community.visibility != community.visibility {
          let actor = self.actor.dereference(context).await?;
          let form = ModlogInsertForm::mod_change_community_visibility(actor.id, old_community.id);
          Modlog::create(&mut context.pool(), &[form]).await?;
        }
      }
      Either::Right(m) => {
        ApubMultiCommunity::from_json(m.clone(), context).await?;
      }
    }
    Ok(())
  }
}
