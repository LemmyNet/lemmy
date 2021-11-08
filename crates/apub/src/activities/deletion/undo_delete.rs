use crate::{
  activities::{
    community::{announce::GetCommunity, send_to_community},
    deletion::{receive_delete_action, verify_delete_activity, DeletableObjects},
    generate_activity_id,
    verify_activity,
    verify_is_public,
  },
  activity_lists::AnnouncableActivities,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::deletion::{delete::Delete, undo_delete::UndoDelete},
};
use activitystreams::{activity::kind::UndoType, public};
use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  traits::{ActivityHandler, ActorType},
};
use lemmy_db_schema::source::{comment::Comment, community::Community, post::Post};
use lemmy_utils::LemmyError;
use lemmy_websocket::{
  send::{send_comment_ws_message_simple, send_community_ws_message, send_post_ws_message},
  LemmyContext,
  UserOperationCrud,
};
use url::Url;

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoDelete {
  type DataType = LemmyContext;
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to, &self.cc)?;
    verify_activity(&self.id, self.actor.inner(), &context.settings())?;
    self.object.verify(context, request_counter).await?;
    let community = self.get_community(context, request_counter).await?;
    verify_delete_activity(
      &self.object.object,
      &self.actor,
      &community,
      self.object.summary.is_some(),
      context,
      request_counter,
    )
    .await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    if self.object.summary.is_some() {
      UndoDelete::receive_undo_remove_action(&self.object.object, context).await
    } else {
      receive_delete_action(
        &self.object.object,
        &self.actor,
        false,
        context,
        request_counter,
      )
      .await
    }
  }
}

impl UndoDelete {
  pub(in crate::activities::deletion) async fn send(
    actor: &ApubPerson,
    community: &ApubCommunity,
    object_id: Url,
    summary: Option<String>,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let object = Delete::new(actor, community, object_id, summary, context)?;

    let id = generate_activity_id(
      UndoType::Undo,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let undo = UndoDelete {
      actor: ObjectId::new(actor.actor_id()),
      to: vec![public()],
      object,
      cc: vec![community.actor_id()],
      kind: UndoType::Undo,
      id: id.clone(),
      unparsed: Default::default(),
    };

    let activity = AnnouncableActivities::UndoDelete(undo);
    send_to_community(activity, &id, actor, community, vec![], context).await
  }

  pub(in crate::activities) async fn receive_undo_remove_action(
    object: &Url,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    use UserOperationCrud::*;
    match DeletableObjects::read_from_db(object, context).await? {
      DeletableObjects::Community(community) => {
        if community.local {
          return Err(anyhow!("Only local admin can restore community").into());
        }
        let deleted_community = blocking(context.pool(), move |conn| {
          Community::update_removed(conn, community.id, false)
        })
        .await??;
        send_community_ws_message(deleted_community.id, EditCommunity, None, None, context).await?;
      }
      DeletableObjects::Post(post) => {
        let removed_post = blocking(context.pool(), move |conn| {
          Post::update_removed(conn, post.id, false)
        })
        .await??;
        send_post_ws_message(removed_post.id, EditPost, None, None, context).await?;
      }
      DeletableObjects::Comment(comment) => {
        let removed_comment = blocking(context.pool(), move |conn| {
          Comment::update_removed(conn, comment.id, false)
        })
        .await??;
        send_comment_ws_message_simple(removed_comment.id, EditComment, context).await?;
      }
    }
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl GetCommunity for UndoDelete {
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    self.object.get_community(context, request_counter).await
  }
}
