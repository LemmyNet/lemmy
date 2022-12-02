use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_person},
  fetcher::user_or_community::UserOrCommunity,
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::following::{follow::Follow, undo_follow::UndoFollow},
  ActorType,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  traits::{ActivityHandler, Actor},
  utils::verify_urls_match,
};
use activitystreams_kinds::activity::UndoType;
use lemmy_db_schema::{
  source::{
    community::{CommunityFollower, CommunityFollowerForm},
    person::{PersonFollower, PersonFollowerForm},
  },
  traits::Followable,
};
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

impl UndoFollow {
  #[tracing::instrument(skip_all)]
  pub async fn send(
    actor: &ApubPerson,
    community: &ApubCommunity,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let object = Follow::new(actor, community, context)?;
    let undo = UndoFollow {
      actor: ObjectId::new(actor.actor_id()),
      object,
      kind: UndoType::Undo,
      id: generate_activity_id(
        UndoType::Undo,
        &context.settings().get_protocol_and_hostname(),
      )?,
    };
    let inbox = vec![community.shared_inbox_or_inbox()];
    send_lemmy_activity(context, undo, actor, inbox, true).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoFollow {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_urls_match(self.actor.inner(), self.object.actor.inner())?;
    verify_person(&self.actor, context, request_counter).await?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let person = self
      .actor
      .dereference(context, local_instance(context).await, request_counter)
      .await?;
    let object = self
      .object
      .object
      .dereference(context, local_instance(context).await, request_counter)
      .await?;

    match object {
      UserOrCommunity::User(u) => {
        let form = PersonFollowerForm {
          person_id: u.id,
          follower_id: person.id,
          pending: false,
        };
        PersonFollower::unfollow(context.pool(), &form).await?;
      }
      UserOrCommunity::Community(c) => {
        let form = CommunityFollowerForm {
          community_id: c.id,
          person_id: person.id,
          pending: false,
        };
        CommunityFollower::unfollow(context.pool(), &form).await?;
      }
    }

    Ok(())
  }
}
