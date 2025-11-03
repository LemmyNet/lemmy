use crate::{
  generate_activity_id,
  protocol::following::{follow::Follow, undo_follow::UndoFollow},
  send_lemmy_activity,
};
use activitypub_federation::{
  config::Data,
  kinds::activity::UndoType,
  protocol::verification::verify_urls_match,
  traits::{Activity, Actor, Object},
};
use either::Either::*;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::objects::{CommunityOrMulti, person::ApubPerson};
use lemmy_db_schema::{
  source::{
    activity::ActivitySendTargets,
    community::CommunityActions,
    community_community_follow::CommunityCommunityFollow,
    instance::InstanceActions,
    multi_community::MultiCommunity,
    person::PersonActions,
  },
  traits::Followable,
};
use lemmy_utils::error::{LemmyError, LemmyResult, UntranslatedError};
use url::Url;

impl UndoFollow {
  pub async fn send(
    actor: &ApubPerson,
    target: &CommunityOrMulti,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let object = Follow::new(actor, target, context)?;
    let undo = UndoFollow {
      actor: actor.id().clone().into(),
      to: Some([target.id().clone().into()]),
      object,
      kind: UndoType::Undo,
      id: generate_activity_id(UndoType::Undo, context)?,
    };
    let inbox = ActivitySendTargets::to_inbox(target.shared_inbox_or_inbox());
    send_lemmy_activity(context, undo, actor, inbox, true).await
  }
}

#[async_trait::async_trait]
impl Activity for UndoFollow {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    verify_urls_match(self.actor.inner(), self.object.actor.inner())?;
    self.object.verify(context).await?;
    if let Some(to) = &self.to {
      verify_urls_match(to[0].inner(), self.object.object.inner())?;
    }
    Ok(())
  }

  async fn receive(self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    let actor = self.actor.dereference(context).await?;
    let object = self.object.object.dereference(context).await?;

    // Handle remote community unfollowing a local community
    if let (Right(community), Right(Left(follower))) = (&actor, &object) {
      CommunityCommunityFollow::unfollow(&mut context.pool(), community.id, follower.id).await?;
      return Ok(());
    }

    let person = actor.left().ok_or(UntranslatedError::InvalidFollow(
      "Groups can only follow public groups".to_string(),
    ))?;
    InstanceActions::check_ban(&mut context.pool(), person.id, person.instance_id).await?;

    match object {
      Left(u) => {
        PersonActions::unfollow(&mut context.pool(), person.id, u.id).await?;
      }
      Right(Left(c)) => {
        CommunityActions::unfollow(&mut context.pool(), person.id, c.id).await?;
      }
      Right(Right(m)) => MultiCommunity::unfollow(&mut context.pool(), person.id, m.id).await?,
    }

    Ok(())
  }
}
