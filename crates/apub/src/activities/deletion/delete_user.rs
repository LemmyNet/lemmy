use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_is_public, verify_person},
  local_instance,
  objects::{instance::remote_instance_inboxes, person::ApubPerson},
  protocol::activities::deletion::delete_user::DeleteUser,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  traits::ActivityHandler,
  utils::verify_urls_match,
};
use activitystreams_kinds::{activity::DeleteType, public};
use lemmy_api_common::utils::delete_user_account;
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

/// This can be separate from Delete activity because it doesn't need to be handled in shared inbox
/// (cause instance actor doesn't have shared inbox).
#[async_trait::async_trait(?Send)]
impl ActivityHandler for DeleteUser {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to, &[])?;
    verify_person(&self.actor, context, request_counter).await?;
    verify_urls_match(self.actor.inner(), self.object.inner())?;
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor = self
      .actor
      .dereference(context, local_instance(context), request_counter)
      .await?;
    delete_user_account(
      actor.id,
      context.pool(),
      context.settings(),
      context.client(),
    )
    .await?;
    Ok(())
  }
}

impl DeleteUser {
  #[tracing::instrument(skip_all)]
  pub async fn send(actor: &ApubPerson, context: &LemmyContext) -> Result<(), LemmyError> {
    let actor_id = ObjectId::new(actor.actor_id.clone());
    let id = generate_activity_id(
      DeleteType::Delete,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let delete = DeleteUser {
      actor: actor_id.clone(),
      to: vec![public()],
      object: actor_id,
      kind: DeleteType::Delete,
      id: id.clone(),
      cc: vec![],
    };

    let inboxes = remote_instance_inboxes(context.pool()).await?;
    send_lemmy_activity(context, delete, actor, inboxes, true).await?;
    Ok(())
  }
}
