use super::send_activity_from_user_or_community_or_multi;
use crate::{
  generate_activity_id,
  protocol::{
    IdOrNestedObject,
    following::{follow::Follow, reject::RejectFollow},
  },
  send_lemmy_activity,
};
use activitypub_federation::{
  config::Data,
  kinds::activity::RejectType,
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

impl RejectFollow {
  pub async fn send(
    follow: Follow,
    context: &Data<LemmyContext>,
    mod_: Option<Person>,
  ) -> LemmyResult<()> {
    let follower = follow.actor.clone().dereference(context).await?;
    match follow.object.dereference(context).await? {
      // Remote mod can't sign as the community, so it sends the reject to the community's instance
      Right(Left(community)) if !community.local => {
        let mod_: ApubPerson = mod_.ok_or(UntranslatedError::Unreachable)?.into();
        let reject = RejectFollow {
          actor: mod_.id().clone().into(),
          to: Some([follower.id().clone().into()]),
          object: IdOrNestedObject::NestedObject(follow),
          kind: RejectType::Reject,
          id: generate_activity_id(RejectType::Reject, context)?,
        };
        let inbox = ActivitySendTargets::to_inbox(community.inbox_url.clone().into());
        send_lemmy_activity(context, reject, &mod_, inbox, true).await
      }
      _ => {
        let user_or_community = follow.object.dereference_local(context).await?;
        let reject = RejectFollow {
          actor: user_or_community.id().clone().into(),
          to: Some([follower.id().clone().into()]),
          object: IdOrNestedObject::NestedObject(follow),
          kind: RejectType::Reject,
          id: generate_activity_id(RejectType::Reject, context)?,
        };
        let inbox = ActivitySendTargets::to_inbox(follower.shared_inbox_or_inbox());
        send_activity_from_user_or_community_or_multi(context, reject, user_or_community, inbox)
          .await
      }
    }
  }
}

/// Handle rejected follows
#[async_trait::async_trait]
impl Activity for RejectFollow {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    let object = self.object.dereference(context).await?;
    match self.actor.dereference(context).await? {
      // Community rejecting a follow of itself
      Right(_) => verify_urls_match(self.actor.inner(), object.object.inner())?,
      // Person rejecting a follow of themselves
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
        // Only the instance hosting the community can forward the reject as the community
        if !community.local {
          return Err(UntranslatedError::InvalidFollow("Not a local object".to_string()).into());
        }
        check_community_deleted_removed(&community)?;
        // remove the follow
        CommunityActions::unfollow(&mut context.pool(), person.id, community.id).await?;
        // Forward remote mod reject to prospective user instance, with the community as actor
        RejectFollow::send(object, context, None).await?;
      }
      Right(community) => {
        check_community_deleted_removed(&community)?;
        // remove the follow
        CommunityActions::unfollow(&mut context.pool(), person.id, community.id).await?;
      }
    }
    Ok(())
  }
}
