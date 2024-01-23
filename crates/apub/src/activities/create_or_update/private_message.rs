use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_person},
  insert_received_activity,
  objects::{person::ApubPerson, private_message::ApubPrivateMessage},
  protocol::activities::{
    create_or_update::chat_message::CreateOrUpdateChatMessage,
    CreateOrUpdateType,
  },
};
use activitypub_federation::{
  config::Data,
  protocol::verification::verify_domains_match,
  traits::{ActivityHandler, Actor, Object},
};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::activity::ActivitySendTargets;
use lemmy_db_views::structs::PrivateMessageView;
use lemmy_utils::error::LemmyError;
use url::Url;

pub(crate) async fn send_create_or_update_pm(
  pm_view: PrivateMessageView,
  kind: CreateOrUpdateType,
  context: Data<LemmyContext>,
) -> Result<(), LemmyError> {
  let actor: ApubPerson = pm_view.creator.into();
  let recipient: ApubPerson = pm_view.recipient.into();

  let id = generate_activity_id(
    kind.clone(),
    &context.settings().get_protocol_and_hostname(),
  )?;
  let create_or_update = CreateOrUpdateChatMessage {
    id: id.clone(),
    actor: actor.id().into(),
    to: [recipient.id().into()],
    object: ApubPrivateMessage(pm_view.private_message.clone())
      .into_json(&context)
      .await?,
    kind,
  };
  let inbox = ActivitySendTargets::to_inbox(recipient.shared_inbox_or_inbox());
  //send_lemmy_activity(&context, create_or_update, &actor, inbox, true).await
  let _ = &context;
  let _ = create_or_update;
  let _ = &actor;
  let _ = inbox;
  Ok(())
}

#[async_trait::async_trait]
impl ActivityHandler for CreateOrUpdateChatMessage {
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
    verify_person(&self.actor, context).await?;
    verify_domains_match(self.actor.inner(), self.object.id.inner())?;
    verify_domains_match(self.to[0].inner(), self.object.to[0].inner())?;
    ApubPrivateMessage::verify(&self.object, self.actor.inner(), context).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<Self::DataType>) -> Result<(), LemmyError> {
    insert_received_activity(&self.id, context).await?;
    ApubPrivateMessage::from_json(self.object, context).await?;
    Ok(())
  }
}
