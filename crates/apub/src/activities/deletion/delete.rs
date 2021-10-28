use crate::{
  activities::{
    community::{
      announce::{AnnouncableActivities, GetCommunity},
      send_to_community,
    },
    deletion::{
      receive_delete_action,
      verify_delete_activity,
      DeletableObjects,
      WebsocketMessages,
    },
    generate_activity_id,
    verify_activity,
    verify_is_public,
  },
  context::lemmy_context,
  fetcher::object_id::ObjectId,
  objects::{community::ApubCommunity, person::ApubPerson},
};
use activitystreams::{
  activity::kind::DeleteType,
  base::AnyBase,
  primitives::OneOrMany,
  public,
  unparsed::Unparsed,
};
use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  data::Data,
  traits::{ActivityFields, ActivityHandler, ActorType},
};
use lemmy_db_schema::{
  source::{
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
    post::Post,
  },
  traits::Crud,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::{
  send::{send_comment_ws_message_simple, send_community_ws_message, send_post_ws_message},
  LemmyContext,
  UserOperationCrud,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

/// This is very confusing, because there are four distinct cases to handle:
/// - user deletes their post
/// - user deletes their comment
/// - remote community mod deletes local community
/// - remote community deletes itself (triggered by a mod)
///
/// TODO: we should probably change how community deletions work to simplify this. Probably by
/// wrapping it in an announce just like other activities, instead of having the community send it.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct Delete {
  actor: ObjectId<ApubPerson>,
  to: Vec<Url>,
  pub(in crate::activities::deletion) object: Url,
  pub(in crate::activities::deletion) cc: Vec<Url>,
  #[serde(rename = "type")]
  kind: DeleteType,
  /// If summary is present, this is a mod action (Remove in Lemmy terms). Otherwise, its a user
  /// deleting their own content.
  pub(in crate::activities::deletion) summary: Option<String>,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for Delete {
  type DataType = LemmyContext;
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to)?;
    verify_activity(self, &context.settings())?;
    let community = self.get_community(context, request_counter).await?;
    verify_delete_activity(
      &self.object,
      self,
      &community,
      self.summary.is_some(),
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
    if let Some(reason) = self.summary {
      // We set reason to empty string if it doesn't exist, to distinguish between delete and
      // remove. Here we change it back to option, so we don't write it to db.
      let reason = if reason.is_empty() {
        None
      } else {
        Some(reason)
      };
      receive_remove_action(&self.actor, &self.object, reason, context, request_counter).await
    } else {
      receive_delete_action(
        &self.object,
        &self.actor,
        WebsocketMessages {
          community: UserOperationCrud::DeleteCommunity,
          post: UserOperationCrud::DeletePost,
          comment: UserOperationCrud::DeleteComment,
        },
        true,
        context,
        request_counter,
      )
      .await
    }
  }
}

impl Delete {
  pub(in crate::activities::deletion) fn new(
    actor: &ApubPerson,
    community: &ApubCommunity,
    object_id: Url,
    summary: Option<String>,
    context: &LemmyContext,
  ) -> Result<Delete, LemmyError> {
    Ok(Delete {
      actor: ObjectId::new(actor.actor_id()),
      to: vec![public()],
      object: object_id,
      cc: vec![community.actor_id()],
      kind: DeleteType::Delete,
      summary,
      id: generate_activity_id(
        DeleteType::Delete,
        &context.settings().get_protocol_and_hostname(),
      )?,
      context: lemmy_context(),
      unparsed: Default::default(),
    })
  }
  pub(in crate::activities::deletion) async fn send(
    actor: &ApubPerson,
    community: &ApubCommunity,
    object_id: Url,
    summary: Option<String>,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let delete = Delete::new(actor, community, object_id, summary, context)?;
    let delete_id = delete.id.clone();

    let activity = AnnouncableActivities::Delete(delete);
    send_to_community(activity, &delete_id, actor, community, vec![], context).await
  }
}

pub(in crate::activities) async fn receive_remove_action(
  actor: &ObjectId<ApubPerson>,
  object: &Url,
  reason: Option<String>,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let actor = actor.dereference(context, request_counter).await?;
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

      send_community_ws_message(deleted_community.id, RemoveCommunity, None, None, context).await?;
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

      send_post_ws_message(removed_post.id, RemovePost, None, None, context).await?;
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

      send_comment_ws_message_simple(removed_comment.id, RemoveComment, context).await?;
    }
  }
  Ok(())
}

#[async_trait::async_trait(?Send)]
impl GetCommunity for Delete {
  async fn get_community(
    &self,
    context: &LemmyContext,
    _request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let community_id = match DeletableObjects::read_from_db(&self.object, context).await? {
      DeletableObjects::Community(c) => c.id,
      DeletableObjects::Comment(c) => {
        let post = blocking(context.pool(), move |conn| Post::read(conn, c.post_id)).await??;
        post.community_id
      }
      DeletableObjects::Post(p) => p.community_id,
    };
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;
    Ok(community.into())
  }
}
