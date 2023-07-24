use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_person},
  fetcher::user_or_community::UserOrCommunity,
  insert_received_activity,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::following::{follow::Follow, undo_follow::UndoFollow},
};
use activitypub_federation::{
  config::Data,
  kinds::activity::UndoType,
  protocol::verification::verify_urls_match,
  traits::{ActivityHandler, Actor},
};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{
    community::{CommunityFollower, CommunityFollowerForm},
    person::{PersonFollower, PersonFollowerForm},
  },
  traits::Followable,
};
use lemmy_utils::error::LemmyError;
use url::Url;

impl UndoFollow {
  #[tracing::instrument(skip_all)]
  pub async fn send(
    actor: &ApubPerson,
    community: &ApubCommunity,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let object = Follow::new(actor, community, context)?;
    let undo = UndoFollow {
      actor: actor.id().into(),
      to: Some([community.id().into()]),
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

#[async_trait::async_trait]
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
  async fn verify(&self, context: &Data<LemmyContext>) -> Result<(), LemmyError> {
    insert_received_activity(&self.id, context).await?;
    verify_urls_match(self.actor.inner(), self.object.actor.inner())?;
    verify_person(&self.actor, context).await?;
    self.object.verify(context).await?;
    if let Some(to) = &self.to {
      verify_urls_match(to[0].inner(), self.object.object.inner())?;
    }
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<LemmyContext>) -> Result<(), LemmyError> {
    let person = self.actor.dereference(context).await?;
    let object = self.object.object.dereference(context).await?;

    match object {
      UserOrCommunity::User(u) => {
        let form = PersonFollowerForm {
          person_id: u.id,
          follower_id: person.id,
          pending: false,
        };
        PersonFollower::unfollow(&mut context.pool(), form).await?;
      }
      UserOrCommunity::Community(c) => {
        let form = CommunityFollowerForm {
          community_id: c.id,
          person_id: person.id,
          pending: false,
        };
        CommunityFollower::unfollow(&mut context.pool(), form).await?;
      }
    }

    Ok(())
  }
}
