use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_person},
  protocol::activities::following::{follow::Follow, undo_follow::UndoFollow},
};
use activitypub_federation::{
  config::Data,
  kinds::activity::UndoType,
  protocol::verification::verify_urls_match,
  traits::{Activity, Actor, Object},
};
use either::Either::*;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::objects::{person::ApubPerson, CommunityOrMulti};
use lemmy_db_schema::{
  source::{
    activity::ActivitySendTargets,
    community::CommunityActions,
    multi_community::MultiCommunity,
    person::PersonActions,
  },
  traits::Followable,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
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
    verify_person(&self.actor, context).await?;
    self.object.verify(context).await?;
    if let Some(to) = &self.to {
      verify_urls_match(to[0].inner(), self.object.object.inner())?;
    }
    Ok(())
  }

  async fn receive(self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    let person = self.actor.dereference(context).await?;
    let object = self.object.object.dereference(context).await?;

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
