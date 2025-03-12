use crate::{
  activities::{
    generate_activity_id,
    send_lemmy_activity,
    verify_person,
    verify_person_in_community,
  },
  fetcher::user_or_community::UserOrCommunity,
  insert_received_activity,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::following::{accept::AcceptFollow, follow::Follow},
};
use activitypub_federation::{
  config::Data,
  kinds::activity::FollowType,
  protocol::verification::verify_urls_match,
  traits::{ActivityHandler, Actor},
};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{
    activity::ActivitySendTargets,
    community::{CommunityFollower, CommunityFollowerForm, CommunityFollowerState},
    instance::Instance,
    person::{PersonFollower, PersonFollowerForm},
  },
  traits::Followable,
  CommunityVisibility,
};
use lemmy_utils::error::{FederationError, LemmyError, LemmyErrorType, LemmyResult};
use url::Url;

impl Follow {
  pub(in crate::activities::following) fn new(
    actor: &ApubPerson,
    community: &ApubCommunity,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<Follow> {
    Ok(Follow {
      actor: actor.id().into(),
      object: community.id().into(),
      to: Some([community.id().into()]),
      kind: FollowType::Follow,
      id: generate_activity_id(
        FollowType::Follow,
        &context.settings().get_protocol_and_hostname(),
      )?,
    })
  }

  pub async fn send(
    actor: &ApubPerson,
    community: &ApubCommunity,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let follow = Follow::new(actor, community, context)?;
    let inbox = if community.local {
      ActivitySendTargets::empty()
    } else {
      ActivitySendTargets::to_inbox(community.shared_inbox_or_inbox())
    };
    send_lemmy_activity(context, follow, actor, inbox, true).await
  }
}

#[async_trait::async_trait]
impl ActivityHandler for Follow {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    verify_person(&self.actor, context).await?;
    let object = self.object.dereference(context).await?;
    if let UserOrCommunity::Community(c) = object {
      verify_person_in_community(&self.actor, &c, context).await?;
    }
    if let Some(to) = &self.to {
      verify_urls_match(to[0].inner(), self.object.inner())?;
    }
    Ok(())
  }

  async fn receive(self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    insert_received_activity(&self.id, context).await?;
    let actor = self.actor.dereference(context).await?;
    let object = self.object.dereference(context).await?;
    match object {
      UserOrCommunity::User(u) => {
        let form = PersonFollowerForm {
          person_id: u.id,
          follower_id: actor.id,
          pending: false,
        };
        PersonFollower::follow(&mut context.pool(), &form).await?;
        AcceptFollow::send(self, context).await?;
      }
      UserOrCommunity::Community(c) => {
        if c.visibility == CommunityVisibility::Private {
          let instance = Instance::read(&mut context.pool(), actor.instance_id).await?;
          if [Some("kbin"), Some("mbin")].contains(&instance.software.as_deref()) {
            // TODO: change this to a minimum version check once private communities are supported
            return Err(FederationError::PlatformLackingPrivateCommunitySupport.into());
          }
        }
        let state = Some(match c.visibility {
          CommunityVisibility::Public => CommunityFollowerState::Accepted,
          CommunityVisibility::Private => CommunityFollowerState::ApprovalRequired,
          // Dont allow following local-only community via federation.
          CommunityVisibility::LocalOnly => return Err(LemmyErrorType::NotFound.into()),
        });
        let form = CommunityFollowerForm {
          state,
          ..CommunityFollowerForm::new(c.id, actor.id)
        };
        CommunityFollower::follow(&mut context.pool(), &form).await?;
        if c.visibility == CommunityVisibility::Public {
          AcceptFollow::send(self, context).await?;
        }
      }
    }
    Ok(())
  }
}
