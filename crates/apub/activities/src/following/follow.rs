use crate::{
  activity_lists::AnnouncableActivities,
  generate_activity_id,
  protocol::{
    community::announce::AnnounceActivity,
    following::{accept::AcceptFollow, follow::Follow},
  },
  send_lemmy_activity,
};
use activitypub_federation::{
  config::Data,
  kinds::activity::FollowType,
  protocol::verification::verify_urls_match,
  traits::{Activity, Actor, Object},
};
use either::Either::*;
use lemmy_api_utils::{context::LemmyContext, utils::check_community_deleted_removed};
use lemmy_apub_objects::objects::{CommunityOrMulti, person::ApubPerson};
use lemmy_db_schema::{
  source::{
    activity::ActivitySendTargets,
    community::{CommunityActions, CommunityFollowerForm},
    community_community_follow::CommunityCommunityFollow,
    instance::{Instance, InstanceActions},
    multi_community::{MultiCommunity, MultiCommunityFollowForm},
    person::{PersonActions, PersonFollowerForm},
  },
  traits::Followable,
};
use lemmy_db_schema_file::enums::{CommunityFollowerState, CommunityVisibility};
use lemmy_db_views_community_moderator::{CommunityModeratorView, CommunityPersonBanView};
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult, UntranslatedError};
use url::Url;

impl Follow {
  pub(in crate::following) fn new(
    actor: &ApubPerson,
    target: &CommunityOrMulti,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<Follow> {
    Ok(Follow {
      actor: actor.id().clone().into(),
      object: target.id().clone().into(),
      to: Some([target.id().clone().into()]),
      kind: FollowType::Follow,
      id: generate_activity_id(FollowType::Follow, context)?,
    })
  }

  pub async fn send(
    actor: &ApubPerson,
    target: &CommunityOrMulti,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    match target {
      // Remote mods of a local private community need to see the request
      Left(community)
        if community.local && community.visibility == CommunityVisibility::Private =>
      {
        let follow = Follow::new(actor, target, context)?;
        let announce = AnnounceActivity::new(
          AnnouncableActivities::Follow(follow).try_into()?,
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
      // Following any other local community needs no federation
      Left(community) if community.local => Ok(()),
      _ => {
        let follow = Follow::new(actor, target, context)?;
        let inbox = ActivitySendTargets::to_inbox(target.shared_inbox_or_inbox());
        send_lemmy_activity(context, follow, actor, inbox, true).await
      }
    }
  }
}

#[async_trait::async_trait]
impl Activity for Follow {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, _context: &Data<LemmyContext>) -> LemmyResult<()> {
    if let Some(to) = &self.to {
      verify_urls_match(to[0].inner(), self.object.inner())?;
    }
    Ok(())
  }

  async fn receive(self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    use CommunityVisibility::*;
    let actor = self.actor.dereference(context).await?;
    let object = self.object.dereference(context).await?;

    let object_local = match &object {
      Left(u) => u.local,
      Right(Left(c)) => c.local,
      Right(Right(m)) => m.local,
    };

    // A person following a remote community is allowed, so remote mods receive the request
    if !object_local && !matches!((&actor, &object), (Left(_), Right(Left(_)))) {
      return Err(UntranslatedError::InvalidFollow("Not a local object".to_string()).into());
    }

    // Handle remote community following a local community
    if let (Right(community), Right(Left(follower))) = (&actor, &object)
      && (community.visibility == Public || community.visibility == Unlisted)
    {
      check_community_deleted_removed(community)?;
      CommunityCommunityFollow::follow(&mut context.pool(), community.id, follower.id).await?;
      AcceptFollow::send(self, context, None).await?;
      return Ok(());
    }

    let person = actor.left().ok_or(UntranslatedError::InvalidFollow(
      "Groups can only follow public groups".to_string(),
    ))?;
    InstanceActions::check_ban(&mut context.pool(), person.id, person.instance_id).await?;

    match object {
      Left(u) => {
        let form = PersonFollowerForm::new(u.id, person.id, false);
        PersonActions::follow(&mut context.pool(), &form).await?;
        AcceptFollow::send(self, context, None).await?;
      }
      Right(Left(c)) => {
        check_community_deleted_removed(&c)?;
        CommunityPersonBanView::check(&mut context.pool(), person.id, c.id).await?;
        if c.visibility == CommunityVisibility::Private {
          let instance = Instance::read(&mut context.pool(), person.instance_id).await?;
          if [Some("kbin"), Some("mbin")].contains(&instance.software.as_deref()) {
            // TODO: change this to a minimum version check once private communities are supported
            return Err(
              UntranslatedError::InvalidFollow("No private community support".to_string()).into(),
            );
          }
        }

        let follow_state = match c.visibility {
          Public | Unlisted => CommunityFollowerState::Accepted,
          Private => {
            // Auto-accept follow requests from mods (e.g. when a mod unfollows and refollows)
            if CommunityModeratorView::check_is_community_moderator(
              &mut context.pool(),
              c.id,
              person.id,
            )
            .await
            .is_ok()
            {
              CommunityFollowerState::Accepted
            } else {
              CommunityFollowerState::ApprovalRequired
            }
          }
          // Dont allow following local-only community via federation.
          LocalOnlyPrivate | LocalOnlyPublic => return Err(LemmyErrorType::NotFound.into()),
        };
        let form = CommunityFollowerForm {
          follow_activity_id: Some(self.id.clone().into()),
          ..CommunityFollowerForm::new(c.id, person.id, follow_state)
        };
        CommunityActions::follow(&mut context.pool(), &form).await?;

        // Only the instance hosting the community answers the follow, or announces it to mods
        // for approval
        if c.local {
          match follow_state {
            CommunityFollowerState::Accepted => {
              AcceptFollow::send(self, context, None).await?;
            }
            CommunityFollowerState::ApprovalRequired => {
              let announce = AnnounceActivity::new(
                AnnouncableActivities::Follow(self).try_into()?,
                &c,
                None,
                context,
              )?;
              let mut inboxes = ActivitySendTargets::empty();
              let moderators =
                CommunityModeratorView::for_community(&mut context.pool(), c.id).await?;
              for m in moderators {
                inboxes.add_inbox(m.moderator.inbox_url.into());
              }
              send_lemmy_activity(context, announce, &c, inboxes, true).await?;
            }
            _ => {}
          }
        }
      }
      Right(Right(m)) => {
        let form = MultiCommunityFollowForm {
          multi_community_id: m.id,
          person_id: person.id,
          follow_state: CommunityFollowerState::Accepted,
        };

        MultiCommunity::follow(&mut context.pool(), &form).await?;
        AcceptFollow::send(self, context, None).await?;
      }
    }
    Ok(())
  }
}
