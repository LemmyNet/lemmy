use crate::{
  activities::{generate_activity_id, send_lemmy_activity},
  insert_received_activity,
  protocol::activities::following::{accept::AcceptFollow, follow::Follow},
};
use activitypub_federation::{
  config::Data,
  kinds::activity::AcceptType,
  protocol::verification::verify_urls_match,
  traits::{ActivityHandler, Actor},
};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{source::community::CommunityFollower, traits::Followable};
use lemmy_utils::error::LemmyError;
use url::Url;

impl AcceptFollow {
  #[tracing::instrument(skip_all)]
  pub async fn send(follow: Follow, context: &Data<LemmyContext>) -> Result<(), LemmyError> {
    let user_or_community = follow.object.dereference_local(context).await?;
    let person = follow.actor.clone().dereference(context).await?;
    let accept = AcceptFollow {
      actor: user_or_community.id().into(),
      to: Some([person.id().into()]),
      object: follow,
      kind: AcceptType::Accept,
      id: generate_activity_id(
        AcceptType::Accept,
        &context.settings().get_protocol_and_hostname(),
      )?,
    };
    let inbox = vec![person.shared_inbox_or_inbox()];
    send_lemmy_activity(context, accept, &user_or_community, inbox, true).await
  }
}

/// Handle accepted follows
#[async_trait::async_trait]
impl ActivityHandler for AcceptFollow {
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
    verify_urls_match(self.actor.inner(), self.object.object.inner())?;
    self.object.verify(context).await?;
    if let Some(to) = &self.to {
      verify_urls_match(to[0].inner(), self.object.actor.inner())?;
    }
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<LemmyContext>) -> Result<(), LemmyError> {
    let community = self.actor.dereference(context).await?;
    let person = self.object.actor.dereference(context).await?;
    // This will throw an error if no follow was requested
    let community_id = community.id;
    let person_id = person.id;
    CommunityFollower::follow_accepted(&mut context.pool(), community_id, person_id).await?;

    Ok(())
  }
}
