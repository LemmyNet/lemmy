use super::{to, update_removed_for_instance};
use crate::{
  MOD_ACTION_DEFAULT_REASON,
  activity_lists::AnnouncableActivities,
  block::{SiteOrCommunity, generate_cc},
  community::send_activity_in_community,
  generate_activity_id,
  protocol::block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
  send_lemmy_activity,
};
use activitypub_federation::{
  config::Data,
  kinds::activity::UndoType,
  protocol::verification::verify_domains_match,
  traits::{Activity, Actor, Object},
};
use lemmy_api_utils::{
  context::LemmyContext,
  notify::notify_mod_action,
  utils::{remove_or_restore_user_data, remove_or_restore_user_data_in_community},
};
use lemmy_apub_objects::{
  objects::person::ApubPerson,
  utils::functions::{verify_is_public, verify_visibility},
};
use lemmy_db_schema::{
  source::{
    activity::ActivitySendTargets,
    community::{CommunityActions, CommunityPersonBanForm},
    instance::{InstanceActions, InstanceBanForm},
    modlog::{Modlog, ModlogInsertForm},
  },
  traits::Bannable,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl UndoBlockUser {
  pub async fn send(
    target: &SiteOrCommunity,
    user: &ApubPerson,
    mod_: &ApubPerson,
    restore_data: bool,
    reason: String,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let block = BlockUser::new(target, user, mod_, None, reason, None, context).await?;
    let to = to(target)?;

    let id = generate_activity_id(UndoType::Undo, context)?;
    let undo = UndoBlockUser {
      actor: mod_.id().clone().into(),
      to,
      object: block,
      cc: generate_cc(target, &mut context.pool()).await?,
      kind: UndoType::Undo,
      id: id.clone(),
      restore_data: Some(restore_data),
    };

    let mut inboxes = ActivitySendTargets::to_inbox(user.shared_inbox_or_inbox());
    match target {
      SiteOrCommunity::Left(_) => {
        inboxes.set_all_instances();
        send_lemmy_activity(context, undo, mod_, inboxes, false).await
      }
      SiteOrCommunity::Right(c) => {
        let activity = AnnouncableActivities::UndoBlockUser(undo);
        send_activity_in_community(activity, mod_, c, inboxes, true, context).await
      }
    }
  }
}

#[async_trait::async_trait]
impl Activity for UndoBlockUser {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    verify_domains_match(self.actor.inner(), self.object.actor.inner())?;
    self.object.verify(context).await?;
    Ok(())
  }

  async fn receive(self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    let expires_at = self.object.end_time;
    let mod_person = self.actor.dereference(context).await?;
    let blocked_person = self.object.object.dereference(context).await?;
    let reason = self
      .object
      .summary
      .unwrap_or_else(|| MOD_ACTION_DEFAULT_REASON.to_string());
    let pool = &mut context.pool();
    match self.object.target.dereference(context).await? {
      SiteOrCommunity::Left(site) => {
        verify_is_public(&self.to, &self.cc)?;
        let form = InstanceBanForm::new(blocked_person.id, site.instance_id, expires_at);
        InstanceActions::unban(pool, &form).await?;

        if self.restore_data.unwrap_or(false) {
          if blocked_person.instance_id == site.instance_id {
            // user unbanned from home instance, restore all content
            remove_or_restore_user_data(mod_person.id, blocked_person.id, false, &reason, context)
              .await?;
          } else {
            update_removed_for_instance(&blocked_person, &site, false, pool).await?;
          }
        }

        // write mod log
        let form =
          ModlogInsertForm::admin_ban(&mod_person, blocked_person.id, false, expires_at, &reason);
        let action = Modlog::create(&mut context.pool(), &[form]).await?;
        notify_mod_action(action.clone(), context.app_data());
      }
      SiteOrCommunity::Right(community) => {
        verify_visibility(&self.to, &self.cc, &community)?;
        let community_user_ban_form = CommunityPersonBanForm::new(community.id, blocked_person.id);
        CommunityActions::unban(&mut context.pool(), &community_user_ban_form).await?;

        if self.restore_data.unwrap_or(false) {
          remove_or_restore_user_data_in_community(
            community.id,
            mod_person.id,
            blocked_person.id,
            false,
            &reason,
            &mut context.pool(),
          )
          .await?;
        }

        // write to mod log
        let form = ModlogInsertForm::mod_ban_from_community(
          mod_person.id,
          community.id,
          blocked_person.id,
          false,
          expires_at,
          &reason,
        );
        let action = Modlog::create(&mut context.pool(), &[form]).await?;
        notify_mod_action(action.clone(), context.app_data());
      }
    }

    Ok(())
  }
}
