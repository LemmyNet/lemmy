use crate::{
  activities::{
    community::announce::AnnouncableActivities,
    deletion::{
      receive_delete_action,
      verify_delete_activity,
      DeletableObjects,
      WebsocketMessages,
    },
    generate_activity_id,
    verify_activity,
  },
  activity_queue::send_to_community_new,
  extensions::context::lemmy_context,
  fetcher::person::get_or_fetch_and_upsert_person,
  ActorType,
};
use activitystreams::{
  activity::kind::DeleteType,
  base::AnyBase,
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use anyhow::anyhow;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{values::PublicUrl, ActivityFields, ActivityHandler};
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
use lemmy_websocket::{
  send::{send_comment_ws_message_simple, send_community_ws_message, send_post_ws_message},
  LemmyContext,
  UserOperationCrud,
};
use serde::{Deserialize, Serialize};
use url::Url;

/// This is very confusing, because there are four distinct cases to handle:
/// - user deletes their post
/// - user deletes their comment
/// - remote community mod deletes local community
/// - remote community deletes itself (triggered by a mod)
///
/// TODO: we should probably change how community deletions work to simplify this. Probably by
/// wrapping it in an announce just like other activities, instead of having the community send it.
#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct Delete {
  actor: Url,
  to: PublicUrl,
  pub(in crate::activities::deletion) object: Url,
  pub(in crate::activities::deletion) cc: [Url; 1],
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
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self)?;
    verify_delete_activity(
      &self.object,
      self,
      &self.cc[0],
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
    actor: &Person,
    community: &Community,
    object_id: Url,
    summary: Option<String>,
  ) -> Result<Delete, LemmyError> {
    Ok(Delete {
      actor: actor.actor_id(),
      to: PublicUrl::Public,
      object: object_id,
      cc: [community.actor_id()],
      kind: DeleteType::Delete,
      summary,
      id: generate_activity_id(DeleteType::Delete)?,
      context: lemmy_context(),
      unparsed: Default::default(),
    })
  }
  pub(in crate::activities::deletion) async fn send(
    actor: &Person,
    community: &Community,
    object_id: Url,
    summary: Option<String>,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let delete = Delete::new(actor, community, object_id, summary)?;
    let delete_id = delete.id.clone();

    let activity = AnnouncableActivities::Delete(delete);
    send_to_community_new(activity, &delete_id, actor, community, vec![], context).await
  }
}

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
