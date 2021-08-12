use crate::{
  activities::{
    comment::send_websocket_message as send_comment_message,
    community::{
      announce::AnnouncableActivities,
      send_websocket_message as send_community_message,
    },
    deletion::{send_apub_delete, verify_delete_activity, DeletableObjects},
    generate_activity_id,
    post::send_websocket_message as send_post_message,
    verify_activity,
  },
  activity_queue::send_to_community_new,
  extensions::context::lemmy_context,
  fetcher::person::get_or_fetch_and_upsert_person,
  ActorType,
};
use activitystreams::activity::kind::DeleteType;
use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{values::PublicUrl, ActivityCommonFields, ActivityHandler};
use lemmy_db_queries::{
  source::{comment::Comment_, community::Community_, post::Post_},
  Crud,
};
use lemmy_db_schema::source::{
  comment::Comment,
  community::Community,
  moderator::{
    ModRemoveComment,
    ModRemoveCommentForm,
    ModRemoveCommunity,
    ModRemoveCommunityForm,
    ModRemovePost,
    ModRemovePostForm,
  },
  person::Person,
  post::Post,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::{LemmyContext, UserOperationCrud};
use url::Url;

/// This is very confusing, because there are four distinct cases to handle:
/// - user deletes their post
/// - user deletes their comment
/// - remote community mod deletes local community
/// - remote community deletes itself (triggered by a mod)
///
/// TODO: we should probably change how community deletions work to simplify this. Probably by
/// wrapping it in an announce just like other activities, instead of having the community send it.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Delete {
  pub(in crate::activities::deletion) to: PublicUrl,
  pub(in crate::activities::deletion) object: Url,
  pub(in crate::activities::deletion) cc: [Url; 1],
  #[serde(rename = "type")]
  pub(in crate::activities::deletion) kind: DeleteType,
  /// If summary is present, this is a mod action (Remove in Lemmy terms). Otherwise, its a user
  /// deleting their own content.
  pub(in crate::activities::deletion) summary: Option<String>,
  #[serde(flatten)]
  pub(in crate::activities::deletion) common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for Delete {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    verify_delete_activity(
      &self.object,
      &self.cc[0],
      &self.common,
      self.summary.is_some(),
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
    if let Some(reason) = self.summary {
      receive_remove_action(
        &self.common.actor,
        &self.object,
        Some(reason),
        context,
        request_counter,
      )
      .await
    } else {
      self.receive_delete_action(context, request_counter).await
    }
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}

impl Delete {
  pub(in crate::activities::deletion) async fn send(
    actor: &Person,
    community: &Community,
    object_id: Url,
    summary: Option<String>,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let id = generate_activity_id(DeleteType::Delete)?;
    let delete = Delete {
      to: PublicUrl::Public,
      object: object_id,
      cc: [community.actor_id()],
      kind: DeleteType::Delete,
      summary,
      common: ActivityCommonFields {
        context: lemmy_context(),
        id: id.clone(),
        actor: actor.actor_id(),
        unparsed: Default::default(),
      },
    };

    let activity = AnnouncableActivities::Delete(delete);
    send_to_community_new(activity, &id, actor, community, vec![], context).await
  }

  // TODO: would be nice if we could merge this and receive_delete_action() into a single method
  //       (or merge with receive_undo_delete_action() instead)
  async fn receive_delete_action(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    use UserOperationCrud::*;
    match DeletableObjects::read_from_db(&self.object, context).await? {
      DeletableObjects::Community(community) => {
        if community.local {
          let mod_ =
            get_or_fetch_and_upsert_person(&self.common.actor, context, request_counter).await?;
          let object = community.actor_id();
          send_apub_delete(&mod_, &community.clone(), object, true, context).await?;
        }

        let deleted_community = blocking(context.pool(), move |conn| {
          Community::update_deleted(conn, community.id, true)
        })
        .await??;
        send_community_message(deleted_community.id, DeleteCommunity, context).await
      }
      DeletableObjects::Post(post) => {
        let deleted_post = blocking(context.pool(), move |conn| {
          Post::update_deleted(conn, post.id, true)
        })
        .await??;
        send_post_message(deleted_post.id, DeletePost, context).await
      }
      DeletableObjects::Comment(comment) => {
        let deleted_comment = blocking(context.pool(), move |conn| {
          Comment::update_deleted(conn, comment.id, true)
        })
        .await??;
        send_comment_message(deleted_comment.id, vec![], DeleteComment, context).await
      }
    }
  }
}

// TODO: reason is optional for compat with v0.11, make it mandatory after removing the migration
pub(in crate::activities) async fn receive_remove_action(
  actor: &Url,
  object: &Url,
  reason: Option<String>,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let actor = get_or_fetch_and_upsert_person(actor, context, request_counter).await?;
  use UserOperationCrud::*;
  match DeletableObjects::read_from_db(object, context).await? {
    DeletableObjects::Community(community) => {
      if community.local {
        return Err(anyhow!("Only local admin can remove community").into());
      }
      let form = ModRemoveCommunityForm {
        mod_person_id: actor.id,
        community_id: community.id,
        removed: Some(true),
        reason,
        expires: None,
      };
      blocking(context.pool(), move |conn| {
        ModRemoveCommunity::create(conn, &form)
      })
      .await??;
      let deleted_community = blocking(context.pool(), move |conn| {
        Community::update_removed(conn, community.id, true)
      })
      .await??;

      send_community_message(deleted_community.id, RemoveCommunity, context).await
    }
    DeletableObjects::Post(post) => {
      let form = ModRemovePostForm {
        mod_person_id: actor.id,
        post_id: post.id,
        removed: Some(true),
        reason,
      };
      blocking(context.pool(), move |conn| {
        ModRemovePost::create(conn, &form)
      })
      .await??;
      let removed_post = blocking(context.pool(), move |conn| {
        Post::update_removed(conn, post.id, true)
      })
      .await??;
      send_post_message(removed_post.id, RemovePost, context).await
    }
    DeletableObjects::Comment(comment) => {
      let form = ModRemoveCommentForm {
        mod_person_id: actor.id,
        comment_id: comment.id,
        removed: Some(true),
        reason,
      };
      blocking(context.pool(), move |conn| {
        ModRemoveComment::create(conn, &form)
      })
      .await??;
      let removed_comment = blocking(context.pool(), move |conn| {
        Comment::update_removed(conn, comment.id, true)
      })
      .await??;
      send_comment_message(removed_comment.id, vec![], RemoveComment, context).await
    }
  }
}
