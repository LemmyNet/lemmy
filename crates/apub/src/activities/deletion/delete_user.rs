use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_is_public, verify_person},
  insert_received_activity,
  objects::person::ApubPerson,
  protocol::activities::deletion::delete_user::DeleteUser,
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
use lemmy_db_schema::source::{activity::ActivitySendTargets, person::Person};
use lemmy_utils::error::LemmyError;
use url::Url;

pub async fn delete_user(person: Person, context: Data<LemmyContext>) -> Result<(), LemmyError> {
  let actor: ApubPerson = person.into();
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

  let mut inboxes = ActivitySendTargets::empty();
  inboxes.set_all_instances(true);

  send_lemmy_activity(&context, delete, &actor, inboxes, true).await?;
  Ok(())
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
