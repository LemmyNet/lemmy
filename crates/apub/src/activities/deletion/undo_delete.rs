use crate::{
  activities::{
    community::announce::GetCommunity,
    deletion::{receive_delete_action, verify_delete_activity, DeletableObjects},
    generate_activity_id,
    verify_activity,
  },
  objects::community::ApubCommunity,
  protocol::activities::deletion::{delete::Delete, undo_delete::UndoDelete},
};
use activitystreams_kinds::activity::UndoType;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{data::Data, object_id::ObjectId, traits::ActivityHandler};
use lemmy_db_schema::source::{comment::Comment, community::Community, person::Person, post::Post};
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

  #[tracing::instrument(skip_all)]
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(&self.id, self.actor.inner(), &context.settings())?;
    self.object.verify(context, request_counter).await?;
    verify_delete_activity(
      &self.object,
      self.object.summary.is_some(),
      context,
      request_counter,
    )
    .await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    if self.object.summary.is_some() {
      UndoDelete::receive_undo_remove_action(self.object.object.id(), context).await
    } else {
      receive_delete_action(
        self.object.object.id(),
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
  #[tracing::instrument(skip_all)]
  pub(in crate::activities::deletion) fn new(
    actor: &Person,
    object: DeletableObjects,
    to: Url,
    community: Option<&Community>,
    summary: Option<String>,
    context: &LemmyContext,
  ) -> Result<UndoDelete, LemmyError> {
    let object = Delete::new(actor, object, to.clone(), community, summary, context)?;

    let id = generate_activity_id(
      UndoType::Undo,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let cc: Option<Url> = community.map(|c| c.actor_id.clone().into());
    Ok(UndoDelete {
      actor: ObjectId::new(actor.actor_id.clone()),
      to: vec![to],
      object,
      cc: cc.into_iter().collect(),
      kind: UndoType::Undo,
      id,
      unparsed: Default::default(),
    })
  }

  #[tracing::instrument(skip_all)]
  pub(in crate::activities) async fn receive_undo_remove_action(
    object: &Url,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    use UserOperationCrud::*;
    match DeletableObjects::read_from_db(object, context).await? {
      DeletableObjects::Community(community) => {
        if community.local {
          return Err(LemmyError::from_message(
            "Only local admin can restore community",
          ));
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
      DeletableObjects::PrivateMessage(_) => unimplemented!(),
    }
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl GetCommunity for UndoDelete {
  #[tracing::instrument(skip_all)]
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    self.object.get_community(context, request_counter).await
  }
}
