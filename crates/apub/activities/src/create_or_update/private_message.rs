use crate::{
  generate_activity_id,
  protocol::{CreateOrUpdateType, create_or_update::private_message::CreateOrUpdatePrivateMessage},
  send_lemmy_activity,
  verify_person,
};
use activitypub_federation::{
  config::Data,
  protocol::verification::{verify_domains_match, verify_urls_match},
  traits::{Activity, Actor, Object},
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::objects::{person::ApubPerson, private_message::ApubPrivateMessage};
use lemmy_db_schema::source::activity::ActivitySendTargets;
use lemmy_db_views_private_message::PrivateMessageView;
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

pub(crate) async fn send_create_or_update_pm(
  pm_view: PrivateMessageView,
  kind: CreateOrUpdateType,
  context: Data<LemmyContext>,
) -> LemmyResult<()> {
  let actor: ApubPerson = pm_view.creator.into();
  let recipient: ApubPerson = pm_view.recipient.into();

  // get object_id
  let pm = &pm_view.private_message;
  let timestamp = pm.updated_at.unwrap_or(pm.published_at);
  let mut object_id = (*pm.ap_id.0).clone();
  object_id.set_fragment(Some(&timestamp.to_rfc3339()));

  let id = generate_activity_id(kind.clone(), Some(&object_id), &context)?;
  let create_or_update = CreateOrUpdatePrivateMessage {
    id: id.clone(),
    actor: actor.id().clone().into(),
    to: [recipient.id().clone().into()],
    object: ApubPrivateMessage(pm_view.private_message.clone())
      .into_json(&context)
      .await?,
    kind,
  };
  let inbox = ActivitySendTargets::to_inbox(recipient.shared_inbox_or_inbox());
  send_lemmy_activity(&context, create_or_update, &actor, inbox, true).await
}

#[async_trait::async_trait]
impl Activity for CreateOrUpdatePrivateMessage {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    verify_person(&self.actor, context).await?;
    verify_domains_match(self.actor.inner(), self.object.id.inner())?;
    verify_domains_match(self.to[0].inner(), self.object.to[0].inner())?;
    verify_urls_match(self.actor.inner(), self.object.attributed_to.inner())?;
    ApubPrivateMessage::verify(&self.object, self.actor.inner(), context).await?;
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    ApubPrivateMessage::from_json(self.object, context).await?;
    Ok(())
  }
}
