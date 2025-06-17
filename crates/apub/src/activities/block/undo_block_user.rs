use super::{to, update_removed_for_instance};
use crate::{
  activities::{
    block::{generate_cc, SiteOrCommunity},
    community::send_activity_in_community,
    generate_activity_id,
    send_lemmy_activity,
  },
  activity_lists::AnnouncableActivities,
  insert_received_activity,
  protocol::activities::block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
};
use activitypub_federation::{
  config::Data,
  kinds::activity::UndoType,
  protocol::verification::verify_domains_match,
  traits::{ActivityHandler, Actor},
};
use lemmy_api_utils::{
  context::LemmyContext,
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
    mod_log::moderator::{ModBan, ModBanForm, ModBanFromCommunity, ModBanFromCommunityForm},
  },
  traits::{Bannable, Crud},
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl UndoBlockUser {
  pub async fn send(
    target: &SiteOrCommunity,
    user: &ApubPerson,
    mod_: &ApubPerson,
    restore_data: bool,
    reason: Option<String>,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let block = BlockUser::new(target, user, mod_, None, reason, None, context).await?;
    let to = to(target)?;

    let id = generate_activity_id(UndoType::Undo, context)?;
    let undo = UndoBlockUser {
      actor: mod_.id().into(),
      to,
      object: block,
      cc: generate_cc(target, &mut context.pool()).await?,
      kind: UndoType::Undo,
      id: id.clone(),
      restore_data: Some(restore_data),
    };

    let mut inboxes = ActivitySendTargets::to_inbox(user.shared_inbox_or_inbox());
    match target {
      SiteOrCommunity::Site(_) => {
        inboxes.set_all_instances();
        send_lemmy_activity(context, undo, mod_, inboxes, false).await
      }
      SiteOrCommunity::Community(c) => {
        let activity = AnnouncableActivities::UndoBlockUser(undo);
        send_activity_in_community(activity, mod_, c, inboxes, true, context).await
      }
    }
  }
}

#[async_trait::async_trait]
impl ActivityHandler for UndoBlockUser {
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
    insert_received_activity(&self.id, context).await?;
    let expires_at = self.object.end_time;
    let mod_person = self.actor.dereference(context).await?;
    let blocked_person = self.object.object.dereference(context).await?;
    let pool = &mut context.pool();
    match self.object.target.dereference(context).await? {
      SiteOrCommunity::Site(site) => {
        verify_is_public(&self.to, &self.cc)?;
        let form = InstanceBanForm::new(blocked_person.id, site.instance_id, expires_at);
        InstanceActions::unban(pool, &form).await?;

        if self.restore_data.unwrap_or(false) {
          if blocked_person.instance_id == site.instance_id {
            // user unbanned from home instance, restore all content
            remove_or_restore_user_data(mod_person.id, blocked_person.id, false, &None, context)
              .await?;
          } else {
            update_removed_for_instance(&blocked_person, &site, false, pool).await?;
          }
        }

        // write mod log
        let form = ModBanForm {
          mod_person_id: mod_person.id,
          other_person_id: blocked_person.id,
          reason: self.object.summary,
          banned: Some(false),
          expires_at,
          instance_id: site.instance_id,
        };
        ModBan::create(&mut context.pool(), &form).await?;
      }
      SiteOrCommunity::Community(community) => {
        verify_visibility(&self.to, &self.cc, &community)?;
        let community_user_ban_form = CommunityPersonBanForm::new(community.id, blocked_person.id);
        CommunityActions::unban(&mut context.pool(), &community_user_ban_form).await?;

        if self.restore_data.unwrap_or(false) {
          remove_or_restore_user_data_in_community(
            community.id,
            mod_person.id,
            blocked_person.id,
            false,
            &None,
            &mut context.pool(),
          )
          .await?;
        }

        // write to mod log
        let form = ModBanFromCommunityForm {
          mod_person_id: mod_person.id,
          other_person_id: blocked_person.id,
          community_id: community.id,
          reason: self.object.summary,
          banned: Some(false),
          expires_at,
        };
        ModBanFromCommunity::create(&mut context.pool(), &form).await?;
      }
    }

    Ok(())
  }
}
