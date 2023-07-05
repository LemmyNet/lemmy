use crate::{
  activities::{
    block::{generate_cc, SiteOrCommunity},
    community::send_activity_in_community,
    generate_activity_id,
    send_lemmy_activity,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  insert_activity,
  objects::{instance::remote_instance_inboxes, person::ApubPerson},
  protocol::activities::block::block_user::BlockUser,
};
use activitypub_federation::{
  config::Data,
  kinds::{activity::BlockType, public},
  protocol::verification::verify_domains_match,
  traits::{ActivityHandler, Actor},
};
use anyhow::anyhow;
use chrono::NaiveDateTime;
use lemmy_api_common::{
  context::LemmyContext,
  utils::{remove_user_data, remove_user_data_in_community},
};
use lemmy_db_schema::{
  source::{
    community::{
      CommunityFollower,
      CommunityFollowerForm,
      CommunityPersonBan,
      CommunityPersonBanForm,
    },
    moderator::{ModBan, ModBanForm, ModBanFromCommunity, ModBanFromCommunityForm},
    person::{Person, PersonUpdateForm},
  },
  traits::{Bannable, Crud, Followable},
};
use lemmy_utils::{error::LemmyError, utils::time::convert_datetime};
use url::Url;

impl BlockUser {
  pub(in crate::activities::block) async fn new(
    target: &SiteOrCommunity,
    user: &ApubPerson,
    mod_: &ApubPerson,
    remove_data: Option<bool>,
    reason: Option<String>,
    expires: Option<NaiveDateTime>,
    context: &Data<LemmyContext>,
  ) -> Result<BlockUser, LemmyError> {
    let audience = if let SiteOrCommunity::Community(c) = target {
      Some(c.id().into())
    } else {
      None
    };
    Ok(BlockUser {
      actor: mod_.id().into(),
      to: vec![public()],
      object: user.id().into(),
      cc: generate_cc(target, context.pool()).await?,
      target: target.id(),
      kind: BlockType::Block,
      remove_data,
      summary: reason,
      id: generate_activity_id(
        BlockType::Block,
        &context.settings().get_protocol_and_hostname(),
      )?,
      audience,
      expires: expires.map(convert_datetime),
    })
  }

  #[tracing::instrument(skip_all)]
  pub async fn send(
    target: &SiteOrCommunity,
    user: &ApubPerson,
    mod_: &ApubPerson,
    remove_data: bool,
    reason: Option<String>,
    expires: Option<NaiveDateTime>,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let block = BlockUser::new(
      target,
      user,
      mod_,
      Some(remove_data),
      reason,
      expires,
      context,
    )
    .await?;

    match target {
      SiteOrCommunity::Site(_) => {
        let inboxes = remote_instance_inboxes(context.pool()).await?;
        send_lemmy_activity(context, block, mod_, inboxes, false).await
      }
      SiteOrCommunity::Community(c) => {
        let activity = AnnouncableActivities::BlockUser(block);
        let inboxes = vec![user.shared_inbox_or_inbox()];
        send_activity_in_community(activity, mod_, c, inboxes, true, context).await
      }
    }
  }
}

#[async_trait::async_trait]
impl ActivityHandler for BlockUser {
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
    verify_is_public(&self.to, &self.cc)?;
    match self.target.dereference(context).await? {
      SiteOrCommunity::Site(site) => {
        let domain = self.object.inner().domain().expect("url needs domain");
        if context.settings().hostname == domain {
          return Err(
            anyhow!("Site bans from remote instance can't affect user's home instance").into(),
          );
        }
        // site ban can only target a user who is on the same instance as the actor (admin)
        verify_domains_match(&site.id(), self.actor.inner())?;
        verify_domains_match(&site.id(), self.object.inner())?;
      }
      SiteOrCommunity::Community(community) => {
        verify_person_in_community(&self.actor, &community, context).await?;
        verify_mod_action(&self.actor, self.object.inner(), community.id, context).await?;
      }
    }
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<LemmyContext>) -> Result<(), LemmyError> {
    insert_activity(&self.id, &self, false, false, context).await?;
    let expires = self.expires.map(|u| u.naive_local());
    let mod_person = self.actor.dereference(context).await?;
    let blocked_person = self.object.dereference(context).await?;
    let target = self.target.dereference(context).await?;
    match target {
      SiteOrCommunity::Site(_site) => {
        let blocked_person = Person::update(
          context.pool(),
          blocked_person.id,
          &PersonUpdateForm::builder()
            .banned(Some(true))
            .ban_expires(Some(expires))
            .build(),
        )
        .await?;
        if self.remove_data.unwrap_or(false) {
          remove_user_data(
            blocked_person.id,
            context.pool(),
            context.settings(),
            context.client(),
          )
          .await?;
        }

        // write mod log
        let form = ModBanForm {
          mod_person_id: mod_person.id,
          other_person_id: blocked_person.id,
          reason: self.summary,
          banned: Some(true),
          expires,
        };
        ModBan::create(context.pool(), &form).await?;
      }
      SiteOrCommunity::Community(community) => {
        let community_user_ban_form = CommunityPersonBanForm {
          community_id: community.id,
          person_id: blocked_person.id,
          expires: Some(expires),
        };
        CommunityPersonBan::ban(context.pool(), &community_user_ban_form).await?;

        // Also unsubscribe them from the community, if they are subscribed
        let community_follower_form = CommunityFollowerForm {
          community_id: community.id,
          person_id: blocked_person.id,
          pending: false,
        };
        CommunityFollower::unfollow(context.pool(), &community_follower_form)
          .await
          .ok();

        if self.remove_data.unwrap_or(false) {
          remove_user_data_in_community(community.id, blocked_person.id, context.pool()).await?;
        }

        // write to mod log
        let form = ModBanFromCommunityForm {
          mod_person_id: mod_person.id,
          other_person_id: blocked_person.id,
          community_id: community.id,
          reason: self.summary,
          banned: Some(true),
          expires,
        };
        ModBanFromCommunity::create(context.pool(), &form).await?;
      }
    }

    Ok(())
  }
}
