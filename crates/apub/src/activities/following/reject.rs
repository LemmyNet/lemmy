use crate::{
  activities::{generate_activity_id, send_lemmy_activity},
  insert_received_activity,
  protocol::activities::following::{follow::Follow, reject::RejectFollow},
};
use activitypub_federation::{
  config::Data,
  kinds::activity::RejectType,
  protocol::verification::verify_urls_match,
  traits::{ActivityHandler, Actor},
};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{
    activity::ActivitySendTargets,
    community::{CommunityFollower, CommunityFollowerForm},
  },
  traits::Followable,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl RejectFollow {
  #[tracing::instrument(skip_all)]
  pub async fn send(follow: Follow, context: &Data<LemmyContext>) -> LemmyResult<()> {
    let user_or_community = follow.object.dereference_local(context).await?;
    let person = follow.actor.clone().dereference(context).await?;
    let reject = RejectFollow {
      actor: user_or_community.id().into(),
      to: Some([person.id().into()]),
      object: follow,
      kind: RejectType::Reject,
      id: generate_activity_id(
        RejectType::Reject,
        &context.settings().get_protocol_and_hostname(),
      )?,
    };
    let inbox = ActivitySendTargets::to_inbox(person.shared_inbox_or_inbox());
    send_lemmy_activity(context, reject, &user_or_community, inbox, true).await
  }
}

/// Handle rejected follows
#[async_trait::async_trait]
impl ActivityHandler for RejectFollow {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(&self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    verify_urls_match(self.actor.inner(), self.object.object.inner())?;
    self.object.verify(context).await?;
    if let Some(to) = &self.to {
      verify_urls_match(to[0].inner(), self.object.actor.inner())?;
    }
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    insert_received_activity(&self.id, context).await?;
    let community = self.actor.dereference(context).await?;
    let person = self.object.actor.dereference(context).await?;

    // remove the follow
    let form = CommunityFollowerForm::new(community.id, person.id);
    CommunityFollower::unfollow(&mut context.pool(), &form).await?;

    Ok(())
  }
}
