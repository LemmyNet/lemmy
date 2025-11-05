use crate::{
  generate_activity_id,
  protocol::following::{accept::AcceptFollow, follow::Follow},
  send_lemmy_activity,
};
use activitypub_federation::{
  config::Data,
  kinds::activity::FollowType,
  protocol::verification::verify_urls_match,
  traits::{Activity, Actor, Object},
};
use either::Either::*;
use lemmy_api_utils::context::LemmyContext;
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
use lemmy_db_views_community_moderator::CommunityPersonBanView;
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
    let follow = Follow::new(actor, target, context)?;
    let inbox = ActivitySendTargets::to_inbox(target.shared_inbox_or_inbox());
    send_lemmy_activity(context, follow, actor, inbox, true).await
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
    if !object_local {
      return Err(UntranslatedError::InvalidFollow("Not a local object".to_string()).into());
    }

    // Handle remote community following a local community
    if let (Right(community), Right(Left(follower))) = (&actor, &object)
      && (community.visibility == Public || community.visibility == Unlisted)
    {
      CommunityCommunityFollow::follow(&mut context.pool(), community.id, follower.id).await?;
      AcceptFollow::send(self, context).await?;
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
        AcceptFollow::send(self, context).await?;
      }
      Right(Left(c)) => {
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
          Private => CommunityFollowerState::ApprovalRequired,
          // Dont allow following local-only community via federation.
          LocalOnlyPrivate | LocalOnlyPublic => return Err(LemmyErrorType::NotFound.into()),
        };
        let form = CommunityFollowerForm::new(c.id, person.id, follow_state);
        CommunityActions::follow(&mut context.pool(), &form).await?;
        if c.visibility == CommunityVisibility::Public {
          AcceptFollow::send(self, context).await?;
        }
      }
      Right(Right(m)) => {
        let form = MultiCommunityFollowForm {
          multi_community_id: m.id,
          person_id: person.id,
          follow_state: CommunityFollowerState::Accepted,
        };

        MultiCommunity::follow(&mut context.pool(), &form).await?;
        AcceptFollow::send(self, context).await?;
      }
    }
    Ok(())
  }
}
