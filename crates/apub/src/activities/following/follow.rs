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
    community::{CommunityFollower, CommunityFollowerForm},
    person::{PersonFollower, PersonFollowerForm},
  },
  traits::Followable,
};
use lemmy_utils::error::LemmyError;
use url::Url;

impl Follow {
  pub(in crate::activities::following) fn new(
    actor: &ApubPerson,
    community: &ApubCommunity,
    context: &Data<LemmyContext>,
  ) -> Result<Follow, LemmyError> {
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

  #[tracing::instrument(skip_all)]
  pub async fn send(
    actor: &ApubPerson,
    community: &ApubCommunity,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let community_follower_form = CommunityFollowerForm {
      community_id: community.id,
      person_id: actor.id,
      pending: true,
    };
    CommunityFollower::follow(&mut context.pool(), &community_follower_form)
      .await
      .ok();

    let follow = Follow::new(actor, community, context)?;
    let inbox = vec![community.shared_inbox_or_inbox()];
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

  #[tracing::instrument(skip_all)]
  async fn verify(&self, context: &Data<LemmyContext>) -> Result<(), LemmyError> {
    insert_received_activity(&self.id, context).await?;
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

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<LemmyContext>) -> Result<(), LemmyError> {
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
      }
      UserOrCommunity::Community(c) => {
        let form = CommunityFollowerForm {
          community_id: c.id,
          person_id: actor.id,
          pending: false,
        };
        CommunityFollower::follow(&mut context.pool(), &form).await?;
      }
    }

    AcceptFollow::send(self, context).await
  }
}
