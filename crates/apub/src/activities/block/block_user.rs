use super::{to, update_removed_for_instance};
use crate::{
  activities::{
    block::{generate_cc, SiteOrCommunity},
    community::send_activity_in_community,
    generate_activity_id,
    send_lemmy_activity,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
    verify_visibility,
  },
  activity_lists::AnnouncableActivities,
  insert_received_activity,
  objects::person::ApubPerson,
  protocol::activities::block::block_user::BlockUser,
};
use activitypub_federation::{
  config::Data,
  kinds::activity::BlockType,
  traits::{ActivityHandler, Actor},
};
use chrono::{DateTime, Utc};
use lemmy_api_common::{
  context::LemmyContext,
  utils::{remove_or_restore_user_data, remove_or_restore_user_data_in_community},
};
use lemmy_db_schema::{
  source::{
    activity::ActivitySendTargets,
    community::{CommunityActions, CommunityPersonBanForm},
    instance::{InstanceActions, InstanceBanForm},
    mod_log::moderator::{ModBan, ModBanForm, ModBanFromCommunity, ModBanFromCommunityForm},
  },
  traits::{Bannable, Crud},
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl BlockUser {
  pub(in crate::activities::block) async fn new(
    target: &SiteOrCommunity,
    user: &ApubPerson,
    mod_: &ApubPerson,
    remove_data: Option<bool>,
    reason: Option<String>,
    expires: Option<DateTime<Utc>>,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<BlockUser> {
    let to = to(target)?;
    Ok(BlockUser {
      actor: mod_.id().into(),
      to,
      object: user.id().into(),
      cc: generate_cc(target, &mut context.pool()).await?,
      target: target.id(),
      kind: BlockType::Block,
      remove_data,
      summary: reason,
      id: generate_activity_id(
        BlockType::Block,
        &context.settings().get_protocol_and_hostname(),
      )?,
      end_time: expires,
    })
  }

  pub async fn send(
    target: &SiteOrCommunity,
    user: &ApubPerson,
    mod_: &ApubPerson,
    remove_data: bool,
    reason: Option<String>,
    expires: Option<DateTime<Utc>>,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
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
        let inboxes = ActivitySendTargets::to_all_instances();
        send_lemmy_activity(context, block, mod_, inboxes, false).await
      }
      SiteOrCommunity::Community(c) => {
        let activity = AnnouncableActivities::BlockUser(block);
        let inboxes = ActivitySendTargets::to_inbox(user.shared_inbox_or_inbox());
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

  async fn verify(&self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    match self.target.dereference(context).await? {
      SiteOrCommunity::Site(_site) => {
        verify_is_public(&self.to, &self.cc)?;
      }
      SiteOrCommunity::Community(community) => {
        verify_visibility(&self.to, &self.cc, &community)?;
        verify_person_in_community(&self.actor, &community, context).await?;
        verify_mod_action(&self.actor, &community, context).await?;
      }
    }
    Ok(())
  }

  async fn receive(self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    insert_received_activity(&self.id, context).await?;
    let expires = self.end_time;
    let mod_person = self.actor.dereference(context).await?;
    let blocked_person = self.object.dereference(context).await?;
    let target = self.target.dereference(context).await?;
    let reason = self.summary;
    let pool = &mut context.pool();
    match target {
      SiteOrCommunity::Site(site) => {
        let form = InstanceBanForm::new(blocked_person.id, site.instance_id, expires);
        InstanceActions::ban(pool, &form).await?;

        if self.remove_data.unwrap_or(false) {
          if blocked_person.instance_id == site.instance_id {
            // user banned from home instance, remove all content
            remove_or_restore_user_data(mod_person.id, blocked_person.id, true, &reason, context)
              .await?;
          } else {
            update_removed_for_instance(&blocked_person, &site, true, pool).await?;
          }
        }

        // write mod log
        let form = ModBanForm {
          mod_person_id: mod_person.id,
          other_person_id: blocked_person.id,
          reason,
          banned: Some(true),
          expires,
          instance_id: site.instance_id,
        };
        ModBan::create(&mut context.pool(), &form).await?;
      }
      SiteOrCommunity::Community(community) => {
        let community_user_ban_form = CommunityPersonBanForm {
          ban_expires: Some(expires),
          ..CommunityPersonBanForm::new(community.id, blocked_person.id)
        };
        CommunityActions::ban(&mut context.pool(), &community_user_ban_form).await?;

        // Dont unsubscribe the user so that we can receive a potential unban activity.
        // If we unfollowed the community here, activities from the community would be rejected
        // in [[can_accept_activity_in_community]] in case are no other local followers.

        if self.remove_data.unwrap_or(false) {
          remove_or_restore_user_data_in_community(
            community.id,
            mod_person.id,
            blocked_person.id,
            true,
            &reason,
            &mut context.pool(),
          )
          .await?;
        }

        // write to mod log
        let form = ModBanFromCommunityForm {
          mod_person_id: mod_person.id,
          other_person_id: blocked_person.id,
          community_id: community.id,
          reason,
          banned: Some(true),
          expires,
        };
        ModBanFromCommunity::create(&mut context.pool(), &form).await?;
      }
    }

    Ok(())
  }
}
