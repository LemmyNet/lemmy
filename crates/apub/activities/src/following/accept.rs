use crate::{
  generate_activity_id,
  protocol::{
    IdOrNestedObject,
    following::{accept::AcceptFollow, follow::Follow},
  },
  send_lemmy_activity,
};
use activitypub_federation::{
  config::Data,
  kinds::activity::AcceptType,
  protocol::verification::verify_urls_match,
  traits::{Activity, Actor, Object},
};
use either::Either::{Left, Right};
use lemmy_api_utils::{context::LemmyContext, utils::check_community_deleted_removed};
use lemmy_apub_objects::{objects::person::ApubPerson, utils::protocol::InCommunity};
use lemmy_db_schema::{
  source::{activity::ActivitySendTargets, community::CommunityActions, person::Person},
  traits::Followable,
};
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_utils::error::{LemmyError, LemmyResult, UntranslatedError};
use url::Url;

impl AcceptFollow {
  pub async fn send(
    follow: Follow,
    context: &Data<LemmyContext>,
    mod_: Option<Person>,
  ) -> LemmyResult<()> {
    let follower = follow.actor.clone().dereference(context).await?;
    match follow.object.dereference(context).await? {
      // Remote mod can't sign as the community, so it sends the accept to the community's instance
      Right(Left(community)) if !community.local => {
        let mod_: ApubPerson = mod_.ok_or(UntranslatedError::Unreachable)?.into();
        let accept = AcceptFollow {
          actor: mod_.id().clone().into(),
          to: Some([follower.id().clone().into()]),
          object: IdOrNestedObject::NestedObject(follow),
          kind: AcceptType::Accept,
          id: generate_activity_id(AcceptType::Accept, context)?,
        };
        let inbox = ActivitySendTargets::to_inbox(community.inbox_url.clone().into());
        send_lemmy_activity(context, accept, &mod_, inbox, true).await
      }
      _ => {
        let target = follow.object.dereference_local(context).await?;
        let accept = AcceptFollow {
          actor: target.id().clone().into(),
          to: Some([follower.id().clone().into()]),
          object: IdOrNestedObject::NestedObject(follow),
          kind: AcceptType::Accept,
          id: generate_activity_id(AcceptType::Accept, context)?,
        };
        let inbox = ActivitySendTargets::to_inbox(follower.shared_inbox_or_inbox());
        send_lemmy_activity(context, accept, &target, inbox, true).await
      }
    }
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
    let object = self.object.clone().dereference(context).await?;
    match self.actor.dereference(context).await? {
      // Community accepting a follow of itself
      Right(_) => verify_urls_match(self.actor.inner(), object.object.inner())?,
      // Person accepting a follow of themselves
      Left(_) if self.actor.inner() == object.object.inner() => {}
      // A remote mod can't sign as the community, so the actor must moderate the followed community
      Left(moderator) => {
        let community = object.community(context).await?;
        CommunityModeratorView::check_is_community_moderator(
          &mut context.pool(),
          community.id,
          moderator.id,
        )
        .await?;
      }
    }
    object.verify(context).await?;
    if let Some(to) = &self.to {
      verify_urls_match(to[0].inner(), object.actor.inner())?;
    }
    Ok(())
  }

  async fn receive(self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    let object = self.object.dereference(context).await?;
    let actor = object.actor.dereference(context).await?;
    let person = actor.left().ok_or(UntranslatedError::Unreachable)?;

    match self.actor.dereference(context).await? {
      Left(_moderator) => {
        let community = object.community(context).await?;
        // Only the instance hosting the community can forward the accept as the community
        if !community.local {
          return Err(UntranslatedError::InvalidFollow("Not a local object".to_string()).into());
        }
        check_community_deleted_removed(&community)?;
        // This will throw an error if no follow was requested
        CommunityActions::follow_accepted(&mut context.pool(), community.id, person.id).await?;
        // Forward remote mod accept to prospective user instance, with the community as actor
        AcceptFollow::send(object, context, None).await?;
      }
      Right(community) => {
        check_community_deleted_removed(&community)?;
        // This will throw an error if no follow was requested
        CommunityActions::follow_accepted(&mut context.pool(), community.id, person.id).await?;
      }
    }
    Ok(())
  }
}
