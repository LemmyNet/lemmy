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
use lemmy_apub_objects::utils::protocol::InCommunity;
use lemmy_db_schema::{
  source::{activity::ActivitySendTargets, community::CommunityActions},
  traits::Followable,
};
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_utils::error::{LemmyError, LemmyResult, UntranslatedError};
use url::Url;

impl AcceptFollow {
  pub(in crate::following) fn new(
    actor: &Url,
    follow: Follow,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<AcceptFollow> {
    Ok(AcceptFollow {
      actor: actor.clone().into(),
      to: Some([follow.actor.inner().clone().into()]),
      object: IdOrNestedObject::NestedObject(follow),
      kind: AcceptType::Accept,
      id: generate_activity_id(AcceptType::Accept, context)?,
    })
  }

  pub async fn send(follow: Follow, context: &Data<LemmyContext>) -> LemmyResult<()> {
    let target = follow.object.dereference_local(context).await?;
    let person = follow.actor.clone().dereference(context).await?;
    let accept = AcceptFollow::new(target.id(), follow, context)?;
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
    let object = self.object.clone().dereference(context).await?;
    let actor = self.actor.dereference(context).await?;

    // The reason we need a match arm is that when a remote moderator sends an Accept/Follow to
    // us, the actor (which is the mod) is different than the object to be followed (which is a
    // community), so we need a different checks to cover both cases.
    match actor {
      // Check that the community actor of the Accept/Follow is actually the community we requested
      // to follow.
      Right(_community) => verify_urls_match(self.actor.inner(), object.object.inner())?,
      // Check that the actor of the Accept/Follow is actually a mod of the community the mod claims
      // to have approved a follow to.
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
    let actor = self.actor.dereference(context).await?;
    let object = self.object.dereference(context).await?;

    let follow_actor = object.actor.dereference(context).await?;
    let person = follow_actor.left().ok_or(UntranslatedError::Unreachable)?;
    let community = object.community(context).await?;

    check_community_deleted_removed(&community)?;

    // This will throw an error if no follow was requested
    CommunityActions::follow_accepted(&mut context.pool(), community.id, person.id).await?;

    // When a remote mod accepts, also forward the accept to the follower's instance, signed by the
    // community. We've already checked the actor is a moderator in the verify handler.
    if actor.is_left() {
      AcceptFollow::send(object, context).await?;
    }
    Ok(())
  }
}
