use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_is_public, verify_person},
  local_instance,
  objects::{instance::remote_instance_inboxes, person::ApubPerson},
  protocol::activities::deletion::delete_user::DeleteUser,
  SendActivity,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  traits::ActivityHandler,
  utils::verify_urls_match,
};
use activitystreams_kinds::{activity::DeleteType, public};
use lemmy_api_common::{
  context::LemmyContext,
  person::{DeleteAccount, DeleteAccountResponse},
  utils::{delete_user_account, get_local_user_view_from_jwt},
};
use lemmy_utils::error::LemmyError;
use url::Url;

#[async_trait::async_trait(?Send)]
impl SendActivity for DeleteAccount {
  type Response = DeleteAccountResponse;

  async fn send_activity(
    request: &Self,
    _response: &Self::Response,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let local_user_view =
      get_local_user_view_from_jwt(&request.auth, context.pool(), context.secret()).await?;
    let actor: ApubPerson = local_user_view.person.into();
    delete_user_account(
      actor.id,
      context.pool(),
      context.settings(),
      context.client(),
    )
    .await?;

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
    send_lemmy_activity(context, delete, &actor, inboxes, true).await?;
    Ok(())
  }
}

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
      .dereference(context, local_instance(context).await, request_counter)
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
