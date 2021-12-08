use crate::{
  activities::{verify_mod_action, verify_person_in_community},
  objects::{comment::ApubComment, community::ApubCommunity, person::ApubPerson, post::ApubPost},
  protocol::{
    activities::deletion::{delete::Delete, undo_delete::UndoDelete},
    objects::tombstone::Tombstone,
  },
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{object_id::ObjectId, traits::ApubObject, verify::verify_domains_match};
use lemmy_db_schema::source::{comment::Comment, community::Community, post::Post};
use lemmy_utils::LemmyError;
use lemmy_websocket::{
  send::{send_comment_ws_message_simple, send_community_ws_message, send_post_ws_message},
  LemmyContext,
  UserOperationCrud,
};
use url::Url;

pub mod delete;
pub mod undo_delete;

#[tracing::instrument(skip_all)]
pub async fn send_apub_delete(
  actor: &ApubPerson,
  community: &ApubCommunity,
  object: DeletableObjects,
  deleted: bool,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  if deleted {
    Delete::send(actor, community, object, None, context).await
  } else {
    UndoDelete::send(actor, community, object, None, context).await
  }
}

// TODO: remove reason is actually optional in lemmy. we set an empty string in that case, but its
//       ugly
#[tracing::instrument(skip_all)]
pub async fn send_apub_remove(
  actor: &ApubPerson,
  community: &ApubCommunity,
  object: DeletableObjects,
  reason: String,
  removed: bool,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  if removed {
    Delete::send(actor, community, object, Some(reason), context).await
  } else {
    UndoDelete::send(actor, community, object, Some(reason), context).await
  }
}

pub enum DeletableObjects {
  Community(Box<ApubCommunity>),
  Comment(Box<ApubComment>),
  Post(Box<ApubPost>),
}

impl DeletableObjects {
  #[tracing::instrument(skip_all)]
  pub(crate) async fn read_from_db(
    ap_id: &Url,
    context: &LemmyContext,
  ) -> Result<DeletableObjects, LemmyError> {
    if let Some(c) = ApubCommunity::read_from_apub_id(ap_id.clone(), context).await? {
      return Ok(DeletableObjects::Community(Box::new(c)));
    }
    if let Some(p) = ApubPost::read_from_apub_id(ap_id.clone(), context).await? {
      return Ok(DeletableObjects::Post(Box::new(p)));
    }
    if let Some(c) = ApubComment::read_from_apub_id(ap_id.clone(), context).await? {
      return Ok(DeletableObjects::Comment(Box::new(c)));
    }
    Err(diesel::NotFound.into())
  }

  pub(crate) fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    match self {
      DeletableObjects::Community(c) => c.to_tombstone(),
      DeletableObjects::Comment(c) => c.to_tombstone(),
      DeletableObjects::Post(p) => p.to_tombstone(),
    }
  }
}

#[tracing::instrument(skip_all)]
pub(in crate::activities) async fn verify_delete_activity(
  object: &Url,
  actor: &ObjectId<ApubPerson>,
  community: &ApubCommunity,
  is_mod_action: bool,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let object = DeletableObjects::read_from_db(object, context).await?;
  match object {
    DeletableObjects::Community(community) => {
      if community.local {
        // can only do this check for local community, in remote case it would try to fetch the
        // deleted community (which fails)
        verify_person_in_community(actor, &community, context, request_counter).await?;
      }
      // community deletion is always a mod (or admin) action
      verify_mod_action(actor, &community, context, request_counter).await?;
    }
    DeletableObjects::Post(p) => {
      verify_delete_activity_post_or_comment(
        actor,
        &p.ap_id.clone().into(),
        community,
        is_mod_action,
        context,
        request_counter,
      )
      .await?;
    }
    DeletableObjects::Comment(c) => {
      verify_delete_activity_post_or_comment(
        actor,
        &c.ap_id.clone().into(),
        community,
        is_mod_action,
        context,
        request_counter,
      )
      .await?;
    }
  }
  Ok(())
}

#[tracing::instrument(skip_all)]
async fn verify_delete_activity_post_or_comment(
  actor: &ObjectId<ApubPerson>,
  object_id: &Url,
  community: &ApubCommunity,
  is_mod_action: bool,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  verify_person_in_community(actor, community, context, request_counter).await?;
  if is_mod_action {
    verify_mod_action(actor, community, context, request_counter).await?;
  } else {
    // domain of post ap_id and post.creator ap_id are identical, so we just check the former
    verify_domains_match(actor.inner(), object_id)?;
  }
  Ok(())
}

/// Write deletion or restoring of an object to the database, and send websocket message.
/// TODO: we should do something similar for receive_remove_action(), but its much more complicated
///       because of the mod log
#[tracing::instrument(skip_all)]
async fn receive_delete_action(
  object: &Url,
  actor: &ObjectId<ApubPerson>,
  deleted: bool,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  match DeletableObjects::read_from_db(object, context).await? {
    DeletableObjects::Community(community) => {
      if community.local {
        let mod_ = actor
          .dereference(context, context.client(), request_counter)
          .await?;
        let object = DeletableObjects::Community(community.clone());
        send_apub_delete(&mod_, &community.clone(), object, true, context).await?;
      }

      let community = blocking(context.pool(), move |conn| {
        Community::update_deleted(conn, community.id, deleted)
      })
      .await??;
      send_community_ws_message(
        community.id,
        UserOperationCrud::DeleteCommunity,
        None,
        None,
        context,
      )
      .await?;
    }
    DeletableObjects::Post(post) => {
      if deleted != post.deleted {
        let deleted_post = blocking(context.pool(), move |conn| {
          Post::update_deleted(conn, post.id, deleted)
        })
        .await??;
        send_post_ws_message(
          deleted_post.id,
          UserOperationCrud::DeletePost,
          None,
          None,
          context,
        )
        .await?;
      }
    }
    DeletableObjects::Comment(comment) => {
      if deleted != comment.deleted {
        let deleted_comment = blocking(context.pool(), move |conn| {
          Comment::update_deleted(conn, comment.id, deleted)
        })
        .await??;
        send_comment_ws_message_simple(
          deleted_comment.id,
          UserOperationCrud::DeleteComment,
          context,
        )
        .await?;
      }
    }
  }
  Ok(())
}
