use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_is_public, verify_person},
  objects::person::ApubPerson,
  protocol::activities::deletion::delete_user::DeleteUser,
};
use activitystreams_kinds::{activity::DeleteType, public};
use lemmy_api_common::{blocking, delete_user_account};
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  traits::ActivityHandler,
  verify::verify_urls_match,
};
use lemmy_db_schema::source::site::Site;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;

/// This can be separate from Delete activity because it doesn't need to be handled in shared inbox
/// (cause instance actor doesn't have shared inbox).
#[async_trait::async_trait(?Send)]
impl ActivityHandler for DeleteUser {
  type DataType = LemmyContext;

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
      .dereference(context, context.client(), request_counter)
      .await?;
    delete_user_account(actor.id, context.pool()).await?;
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

    let remote_sites = blocking(context.pool(), Site::read_remote_sites).await??;
    let inboxes = remote_sites
      .into_iter()
      .map(|s| s.inbox_url.into())
      .collect();
    send_lemmy_activity(context, &delete, &id, actor, inboxes, true).await?;
    Ok(())
  }
}
