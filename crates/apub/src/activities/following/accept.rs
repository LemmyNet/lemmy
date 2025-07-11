use crate::{
  activities::{generate_activity_id, send_lemmy_activity},
  protocol::activities::following::{accept::AcceptFollow, follow::Follow},
};
use activitypub_federation::{
  config::Data,
  kinds::activity::AcceptType,
  protocol::verification::verify_urls_match,
  traits::{Activity, Actor, Object},
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{
  source::{activity::ActivitySendTargets, community::CommunityActions},
  traits::Followable,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl AcceptFollow {
  pub async fn send(follow: Follow, context: &Data<LemmyContext>) -> LemmyResult<()> {
    let target = follow.object.dereference_local(context).await?;
    let person = follow.actor.clone().dereference(context).await?;
    let accept = AcceptFollow {
      actor: target.id().clone().into(),
      to: Some([person.id().clone().into()]),
      object: follow,
      kind: AcceptType::Accept,
      id: generate_activity_id(AcceptType::Accept, context)?,
    };
    let inbox = ActivitySendTargets::to_inbox(person.shared_inbox_or_inbox());
    send_lemmy_activity(context, accept, &target, inbox, true).await
  }
}

/// Handle accepted follows
#[async_trait::async_trait]
impl Activity for AcceptFollow {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    verify_urls_match(self.actor.inner(), self.object.object.inner())?;
    self.object.verify(context).await?;
    if let Some(to) = &self.to {
      verify_urls_match(to[0].inner(), self.object.actor.inner())?;
    }
    Ok(())
  }

  async fn receive(self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    let community = self.actor.dereference(context).await?;
    let person = self.object.actor.dereference(context).await?;
    // This will throw an error if no follow was requested
    let community_id = community.id;
    let person_id = person.id;
    CommunityActions::follow_accepted(&mut context.pool(), community_id, person_id).await?;

    Ok(())
  }
}
