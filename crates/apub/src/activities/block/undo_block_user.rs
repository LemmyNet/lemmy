use crate::{
  activities::{
    block::{generate_cc, SiteOrCommunity},
    community::send_activity_in_community,
    generate_activity_id,
    send_lemmy_activity,
    verify_is_public,
  },
  activity_lists::AnnouncableActivities,
  insert_received_activity,
  objects::person::ApubPerson,
  protocol::activities::block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
};
use activitypub_federation::{
  config::Data,
  kinds::{activity::UndoType, public},
  protocol::verification::verify_domains_match,
  traits::{ActivityHandler, Actor},
};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{
    activity::ActivitySendTargets,
    community::{CommunityPersonBan, CommunityPersonBanForm},
    moderator::{ModBan, ModBanForm, ModBanFromCommunity, ModBanFromCommunityForm},
    person::{Person, PersonUpdateForm},
  },
  traits::{Bannable, Crud},
};
use lemmy_utils::error::LemmyError;
use url::Url;

impl UndoBlockUser {
  #[tracing::instrument(skip_all)]
  pub async fn send(
    target: &SiteOrCommunity,
    user: &ApubPerson,
    mod_: &ApubPerson,
    reason: Option<String>,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let block = BlockUser::new(target, user, mod_, None, reason, None, context).await?;
    let audience = if let SiteOrCommunity::Community(c) = target {
      Some(c.id().into())
    } else {
      None
    };

    let id = generate_activity_id(
      UndoType::Undo,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let undo = UndoBlockUser {
      actor: mod_.id().into(),
      to: vec![public()],
      object: block,
      cc: generate_cc(target, &mut context.pool()).await?,
      kind: UndoType::Undo,
      id: id.clone(),
      audience,
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

  #[tracing::instrument(skip_all)]
  async fn verify(&self, context: &Data<LemmyContext>) -> Result<(), LemmyError> {
    verify_is_public(&self.to, &self.cc)?;
    verify_domains_match(self.actor.inner(), self.object.actor.inner())?;
    self.object.verify(context).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<LemmyContext>) -> Result<(), LemmyError> {
    insert_received_activity(&self.id, context).await?;
    let expires = self.object.end_time.map(Into::into);
    let mod_person = self.actor.dereference(context).await?;
    let blocked_person = self.object.object.dereference(context).await?;
    match self.object.target.dereference(context).await? {
      SiteOrCommunity::Site(_site) => {
        let blocked_person = Person::update(
          &mut context.pool(),
          blocked_person.id,
          &PersonUpdateForm {
            banned: Some(false),
            ban_expires: Some(expires),
            ..Default::default()
          },
        )
        .await?;

        // write mod log
        let form = ModBanForm {
          mod_person_id: mod_person.id,
          other_person_id: blocked_person.id,
          reason: self.object.summary,
          banned: Some(false),
          expires,
        };
        ModBan::create(&mut context.pool(), &form).await?;
      }
      SiteOrCommunity::Community(community) => {
        let community_user_ban_form = CommunityPersonBanForm {
          community_id: community.id,
          person_id: blocked_person.id,
          expires: None,
        };
        CommunityPersonBan::unban(&mut context.pool(), &community_user_ban_form).await?;

        // write to mod log
        let form = ModBanFromCommunityForm {
          mod_person_id: mod_person.id,
          other_person_id: blocked_person.id,
          community_id: community.id,
          reason: self.object.summary,
          banned: Some(false),
          expires,
        };
        ModBanFromCommunity::create(&mut context.pool(), &form).await?;
      }
    }

    Ok(())
  }
}
