use crate::{
  activity_lists::AnnouncableActivities,
  generate_activity_id,
  protocol::{
    IdOrNestedObject,
    community::announce::AnnounceActivity,
    following::{follow::Follow, undo_follow::UndoFollow},
  },
  send_lemmy_activity,
};
use activitypub_federation::{
  config::Data,
  kinds::activity::UndoType,
  protocol::verification::verify_urls_match,
  traits::{Activity, Actor, Object},
};
use either::Either::*;
use lemmy_api_utils::{context::LemmyContext, utils::check_community_deleted_removed};
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
use lemmy_db_schema_file::enums::CommunityVisibility;
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_utils::error::{LemmyError, LemmyResult, UntranslatedError};
use url::Url;

impl UndoFollow {
  fn new(
    actor: &ApubPerson,
    target: &CommunityOrMulti,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<UndoFollow> {
    let follow = Follow::new(actor, target, context)?;
    Ok(UndoFollow {
      actor: actor.id().clone().into(),
      to: Some([target.id().clone().into()]),
      object: IdOrNestedObject::NestedObject(follow),
      kind: UndoType::Undo,
      id: generate_activity_id(UndoType::Undo, context)?,
    })
  }

  pub async fn send(
    actor: &ApubPerson,
    target: &CommunityOrMulti,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    match target {
      // Remote mods of a local private community need to unsee the follow request
      Left(community)
        if community.local && community.visibility == CommunityVisibility::Private =>
      {
        let undo = UndoFollow::new(actor, target, context)?;
        let announce = AnnounceActivity::new(
          AnnouncableActivities::UndoFollow(undo).try_into()?,
          community,
          None,
          context,
        )?;
        let mut inboxes = ActivitySendTargets::empty();
        let moderators =
          CommunityModeratorView::for_community(&mut context.pool(), community.id).await?;
        for m in moderators {
          inboxes.add_inbox(m.moderator.inbox_url.into());
        }
        send_lemmy_activity(context, announce, community, inboxes, true).await
      }
      // Unfollowing any other local community needs no federation
      Left(community) if community.local => Ok(()),
      _ => {
        let undo = UndoFollow::new(actor, target, context)?;
        let inbox = ActivitySendTargets::to_inbox(target.shared_inbox_or_inbox());
        send_lemmy_activity(context, undo, actor, inbox, true).await
      }
    }
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
    let object = self.object.dereference(context).await?;
    verify_urls_match(self.actor.inner(), object.actor.inner())?;
    object.verify(context).await?;
    if let Some(to) = &self.to {
      verify_urls_match(to[0].inner(), object.object.inner())?;
    }
    Ok(())
  }

  async fn receive(self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    let actor = self.actor.dereference(context).await?;
    let object = self.object.dereference(context).await?;
    let object = object.object.dereference(context).await?;

    // Handle remote community unfollowing a local community
    if let (Right(community), Right(Left(follower))) = (&actor, &object) {
      check_community_deleted_removed(community)?;
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
        if c.local && c.visibility == CommunityVisibility::Private {
          let announce = AnnounceActivity::new(
            AnnouncableActivities::UndoFollow(self).try_into()?,
            &c,
            None,
            context,
          )?;
          let mut inboxes = ActivitySendTargets::empty();
          let moderators = CommunityModeratorView::for_community(&mut context.pool(), c.id).await?;
          for m in moderators {
            inboxes.add_inbox(m.moderator.inbox_url.into());
          }
          send_lemmy_activity(context, announce, &c, inboxes, true).await?;
        }
      }
      Right(Right(m)) => MultiCommunity::unfollow(&mut context.pool(), person.id, m.id).await?,
    }

    Ok(())
  }
}
