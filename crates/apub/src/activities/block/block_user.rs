use crate::{
  activities::{
    block::{generate_cc, SiteOrCommunity},
    community::{announce::GetCommunity, send_activity_in_community},
    generate_activity_id,
    send_lemmy_activity,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  local_instance,
  objects::{community::ApubCommunity, instance::remote_instance_inboxes, person::ApubPerson},
  protocol::activities::block::block_user::BlockUser,
  ActorType,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  traits::{ActivityHandler, Actor},
  utils::verify_domains_match,
};
use activitystreams_kinds::{activity::BlockType, public};
use anyhow::anyhow;
use chrono::NaiveDateTime;
use lemmy_api_common::utils::{blocking, remove_user_data, remove_user_data_in_community};
use lemmy_db_schema::{
  source::{
    community::{
      CommunityFollower,
      CommunityFollowerForm,
      CommunityPersonBan,
      CommunityPersonBanForm,
    },
    moderator::{ModBan, ModBanForm, ModBanFromCommunity, ModBanFromCommunityForm},
    person::Person,
  },
  traits::{Bannable, Crud, Followable},
};
use lemmy_utils::{error::LemmyError, utils::convert_datetime};
use lemmy_websocket::LemmyContext;
use url::Url;

impl BlockUser {
  pub(in crate::activities::block) async fn new(
    target: &SiteOrCommunity,
    user: &ApubPerson,
    mod_: &ApubPerson,
    remove_data: Option<bool>,
    reason: Option<String>,
    expires: Option<NaiveDateTime>,
    context: &LemmyContext,
  ) -> Result<BlockUser, LemmyError> {
    Ok(BlockUser {
      actor: ObjectId::new(mod_.actor_id()),
      to: vec![public()],
      object: ObjectId::new(user.actor_id()),
      cc: generate_cc(target, context.pool()).await?,
      target: target.id(),
      kind: BlockType::Block,
      remove_data,
      summary: reason,
      id: generate_activity_id(
        BlockType::Block,
        &context.settings().get_protocol_and_hostname(),
      )?,
      expires: expires.map(convert_datetime),
      unparsed: Default::default(),
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
    context: &LemmyContext,
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
        send_activity_in_community(activity, mod_, c, inboxes, context).await
      }
    }
  }
}

#[async_trait::async_trait(?Send)]
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
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to, &self.cc)?;
    match self
      .target
      .dereference(context, local_instance(context), request_counter)
      .await?
    {
      SiteOrCommunity::Site(site) => {
        let domain = self.object.inner().domain().expect("url needs domain");
        if context.settings().hostname == domain {
          return Err(
            anyhow!("Site bans from remote instance can't affect user's home instance").into(),
          );
        }
        // site ban can only target a user who is on the same instance as the actor (admin)
        verify_domains_match(&site.actor_id(), self.actor.inner())?;
        verify_domains_match(&site.actor_id(), self.object.inner())?;
      }
      SiteOrCommunity::Community(community) => {
        verify_person_in_community(&self.actor, &community, context, request_counter).await?;
        verify_mod_action(
          &self.actor,
          self.object.inner(),
          &community,
          context,
          request_counter,
        )
        .await?;
      }
    }
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let expires = self.expires.map(|u| u.naive_local());
    let mod_person = self
      .actor
      .dereference(context, local_instance(context), request_counter)
      .await?;
    let blocked_person = self
      .object
      .dereference(context, local_instance(context), request_counter)
      .await?;
    let target = self
      .target
      .dereference(context, local_instance(context), request_counter)
      .await?;
    match target {
      SiteOrCommunity::Site(_site) => {
        let blocked_person = blocking(context.pool(), move |conn| {
          Person::ban_person(conn, blocked_person.id, true, expires)
        })
        .await??;
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
        blocking(context.pool(), move |conn| ModBan::create(conn, &form)).await??;
      }
      SiteOrCommunity::Community(community) => {
        let community_user_ban_form = CommunityPersonBanForm {
          community_id: community.id,
          person_id: blocked_person.id,
          expires: Some(expires),
        };
        blocking(context.pool(), move |conn| {
          CommunityPersonBan::ban(conn, &community_user_ban_form)
        })
        .await??;

        // Also unsubscribe them from the community, if they are subscribed
        let community_follower_form = CommunityFollowerForm {
          community_id: community.id,
          person_id: blocked_person.id,
          pending: false,
        };
        blocking(context.pool(), move |conn: &'_ _| {
          CommunityFollower::unfollow(conn, &community_follower_form)
        })
        .await?
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
        blocking(context.pool(), move |conn| {
          ModBanFromCommunity::create(conn, &form)
        })
        .await??;
      }
    }

    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl GetCommunity for BlockUser {
  #[tracing::instrument(skip_all)]
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let target = self
      .target
      .dereference(context, local_instance(context), request_counter)
      .await?;
    match target {
      SiteOrCommunity::Community(c) => Ok(c),
      SiteOrCommunity::Site(_) => Err(anyhow!("Calling get_community() on site activity").into()),
    }
  }
}
