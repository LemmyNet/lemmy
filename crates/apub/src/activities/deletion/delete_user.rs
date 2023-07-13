use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_is_public, verify_person},
  insert_received_activity,
  objects::{instance::remote_instance_inboxes, person::ApubPerson},
  protocol::activities::deletion::delete_user::DeleteUser,
  SendActivity,
};
use activitypub_federation::{
  config::Data,
  kinds::{activity::DeleteType, public},
  protocol::verification::verify_urls_match,
  traits::{ActivityHandler, Actor},
};
use lemmy_api_common::{
  context::LemmyContext,
  person::{DeleteAccount, DeleteAccountResponse},
  utils::{delete_user_account, local_user_view_from_jwt},
};
use lemmy_utils::error::LemmyError;
use url::Url;

#[async_trait::async_trait]
impl SendActivity for DeleteAccount {
  type Response = DeleteAccountResponse;

  async fn send_activity(
    request: &Self,
    _response: &Self::Response,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let local_user_view = local_user_view_from_jwt(&request.auth, context).await?;
    let actor: ApubPerson = local_user_view.person.into();
    delete_user_account(
      actor.id,
      &mut context.pool(),
      context.settings(),
      context.client(),
    )
    .await?;

    let id = generate_activity_id(
      DeleteType::Delete,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let delete = DeleteUser {
      actor: actor.id().into(),
      to: vec![public()],
      object: actor.id().into(),
      kind: DeleteType::Delete,
      id: id.clone(),
      cc: vec![],
    };

    let inboxes = remote_instance_inboxes(&mut context.pool()).await?;
    send_lemmy_activity(context, delete, &actor, inboxes, true).await?;
    Ok(())
  }
}

/// This can be separate from Delete activity because it doesn't need to be handled in shared inbox
/// (cause instance actor doesn't have shared inbox).
#[async_trait::async_trait]
impl ActivityHandler for DeleteUser {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), LemmyError> {
    insert_received_activity(&self.id, context).await?;
    verify_is_public(&self.to, &[])?;
    verify_person(&self.actor, context).await?;
    verify_urls_match(self.actor.inner(), self.object.inner())?;
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> Result<(), LemmyError> {
    let actor = self.actor.dereference(context).await?;
    delete_user_account(
      actor.id,
      &mut context.pool(),
      context.settings(),
      context.client(),
    )
    .await?;
    Ok(())
  }
}
