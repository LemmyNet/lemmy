use crate::{
  activities::{
    comment::send_websocket_message as send_comment_message,
    community::{
      announce::AnnouncableActivities,
      send_websocket_message as send_community_message,
    },
    deletion::{delete::Delete, send_apub_delete, verify_delete_activity, DeletableObjects},
    generate_activity_id,
    post::send_websocket_message as send_post_message,
    verify_activity,
  },
  activity_queue::send_to_community_new,
  extensions::context::lemmy_context,
  fetcher::person::get_or_fetch_and_upsert_person,
  ActorType,
};
use activitystreams::activity::kind::{DeleteType, UndoType};
use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{values::PublicUrl, ActivityCommonFields, ActivityHandler};
use lemmy_db_queries::source::{comment::Comment_, community::Community_, post::Post_};
use lemmy_db_schema::source::{comment::Comment, community::Community, person::Person, post::Post};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoDelete {
  to: PublicUrl,
  object: Delete,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: UndoType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoDelete {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    self.object.verify(context, request_counter).await?;
    verify_delete_activity(
      &self.object.object,
      &self.cc[0],
      &self.common,
      self.object.summary.is_some(),
      context,
      request_counter,
    )
    .await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    if self.object.summary.is_some() {
      UndoDelete::receive_undo_remove_action(&self.object.object, context).await
    } else {
      self
        .receive_undo_delete_action(context, request_counter)
        .await
    }
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}

impl UndoDelete {
  pub(in crate::activities::deletion) async fn send(
    actor: &Person,
    community: &Community,
    object_id: Url,
    summary: Option<String>,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let delete = Delete {
      to: PublicUrl::Public,
      object: object_id,
      cc: [community.actor_id()],
      kind: DeleteType::Delete,
      summary,
      common: ActivityCommonFields {
        context: lemmy_context(),
        id: generate_activity_id(DeleteType::Delete)?,
        actor: actor.actor_id(),
        unparsed: Default::default(),
      },
    };

    let id = generate_activity_id(UndoType::Undo)?;
    let undo = UndoDelete {
      to: PublicUrl::Public,
      object: delete,
      cc: [community.actor_id()],
      kind: UndoType::Undo,
      common: ActivityCommonFields {
        context: lemmy_context(),
        id: id.clone(),
        actor: actor.actor_id(),
        unparsed: Default::default(),
      },
    };

    let activity = AnnouncableActivities::UndoDelete(undo);
    send_to_community_new(activity, &id, actor, community, vec![], context).await
  }
  async fn receive_undo_delete_action(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    use UserOperationCrud::*;
    let object = DeletableObjects::read_from_db(&self.object.object, context).await?;
    match object {
      DeletableObjects::Community(community) => {
        if community.local {
          let mod_ =
            get_or_fetch_and_upsert_person(&self.common.actor, context, request_counter).await?;
          send_apub_delete(
            &mod_,
            &community.clone(),
            community.actor_id(),
            false,
            context,
          )
          .await?;
        }

        let deleted_community = blocking(context.pool(), move |conn| {
          Community::update_deleted(conn, community.id, false)
        })
        .await??;
        send_community_message(deleted_community.id, EditCommunity, context).await
      }
      DeletableObjects::Post(post) => {
        let deleted_post = blocking(context.pool(), move |conn| {
          Post::update_deleted(conn, post.id, false)
        })
        .await??;
        send_post_message(deleted_post.id, EditPost, context).await
      }
      DeletableObjects::Comment(comment) => {
        let deleted_comment = blocking(context.pool(), move |conn| {
          Comment::update_deleted(conn, comment.id, false)
        })
        .await??;
        send_comment_message(deleted_comment.id, vec![], EditComment, context).await
      }
    }
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

        send_community_message(deleted_community.id, EditCommunity, context).await
      }
      DeletableObjects::Post(post) => {
        let removed_post = blocking(context.pool(), move |conn| {
          Post::update_removed(conn, post.id, false)
        })
        .await??;
        send_post_message(removed_post.id, EditPost, context).await
      }
      DeletableObjects::Comment(comment) => {
        let removed_comment = blocking(context.pool(), move |conn| {
          Comment::update_removed(conn, comment.id, false)
        })
        .await??;
        send_comment_message(removed_comment.id, vec![], EditComment, context).await
      }
    }
  }
}
