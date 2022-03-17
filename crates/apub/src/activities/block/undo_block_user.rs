use crate::{
  activities::{
    block::{generate_cc, generate_instance_inboxes, SiteOrCommunity},
    community::{announce::GetCommunity, send_activity_in_community},
    generate_activity_id,
    send_lemmy_activity,
    verify_activity,
    verify_is_public,
  },
  activity_lists::AnnouncableActivities,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
};
use activitystreams_kinds::{activity::UndoType, public};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  traits::{ActivityHandler, ActorType},
  verify::verify_domains_match,
};
use lemmy_db_schema::{
  source::{
    community::{CommunityPersonBan, CommunityPersonBanForm},
    moderator::{ModBan, ModBanForm},
    person::Person,
  },
  traits::{Bannable, Crud},
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;

impl UndoBlockUser {
  #[tracing::instrument(skip_all)]
  pub async fn send(
    target: &SiteOrCommunity,
    user: &ApubPerson,
    mod_: &ApubPerson,
    reason: Option<String>,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let block = BlockUser::new(target, user, mod_, None, reason, None, context).await?;

    let id = generate_activity_id(
      UndoType::Undo,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let undo = UndoBlockUser {
      actor: ObjectId::new(mod_.actor_id()),
      to: vec![public()],
      object: block,
      cc: generate_cc(target, context.pool()).await?,
      kind: UndoType::Undo,
      id: id.clone(),
      unparsed: Default::default(),
    };

    let inboxes = vec![user.shared_inbox_or_inbox_url()];
    match target {
      SiteOrCommunity::Site(_) => {
        let inboxes = generate_instance_inboxes(user, context.pool()).await?;
        send_lemmy_activity(context, &undo, &id, mod_, inboxes, false).await
      }
      SiteOrCommunity::Community(c) => {
        let activity = AnnouncableActivities::UndoBlockUser(undo);
        send_activity_in_community(activity, &id, mod_, c, inboxes, context).await
      }
    }
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoBlockUser {
  type DataType = LemmyContext;

  #[tracing::instrument(skip_all)]
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to, &self.cc)?;
    verify_activity(&self.id, self.actor.inner(), &context.settings())?;
    verify_domains_match(self.actor.inner(), self.object.actor.inner())?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let expires = self.object.expires.map(|u| u.naive_local());
    let mod_person = self
      .actor
      .dereference(context, context.client(), request_counter)
      .await?;
    let blocked_person = self
      .object
      .object
      .dereference(context, context.client(), request_counter)
      .await?;
    match self
      .object
      .target
      .dereference(context, context.client(), request_counter)
      .await?
    {
      SiteOrCommunity::Site(_site) => {
        let blocked_person = blocking(context.pool(), move |conn| {
          Person::ban_person(conn, blocked_person.id, false, expires)
        })
        .await??;

        // write mod log
        let form = ModBanForm {
          mod_person_id: mod_person.id,
          other_person_id: blocked_person.id,
          reason: self.object.summary,
          banned: Some(false),
          expires,
        };
        blocking(context.pool(), move |conn| ModBan::create(conn, &form)).await??;
      }
      SiteOrCommunity::Community(community) => {
        let community_user_ban_form = CommunityPersonBanForm {
          community_id: community.id,
          person_id: blocked_person.id,
          expires: None,
        };
        blocking(context.pool(), move |conn: &'_ _| {
          CommunityPersonBan::unban(conn, &community_user_ban_form)
        })
        .await??;

        // write to mod log
        let form = ModBanForm {
          mod_person_id: mod_person.id,
          other_person_id: blocked_person.id,
          reason: self.object.summary,
          banned: Some(false),
          expires,
        };
        blocking(context.pool(), move |conn| ModBan::create(conn, &form)).await??;
      }
    }

    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl GetCommunity for UndoBlockUser {
  #[tracing::instrument(skip_all)]
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    self.object.get_community(context, request_counter).await
  }
}
