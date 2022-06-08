use crate::{
  activities::{
    community::{announce::GetCommunity, send_activity_in_community},
    send_lemmy_activity,
    verify_is_public,
    verify_mod_action,
    verify_person,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  local_instance,
  objects::{
    comment::ApubComment,
    community::ApubCommunity,
    person::ApubPerson,
    post::ApubPost,
    private_message::ApubPrivateMessage,
  },
  protocol::activities::deletion::{delete::Delete, undo_delete::UndoDelete},
  ActorType,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  traits::{Actor, ApubObject},
  utils::verify_domains_match,
};
use activitystreams_kinds::public;
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{
  source::{
    comment::Comment,
    community::Community,
    person::Person,
    post::Post,
    private_message::PrivateMessage,
  },
  traits::Crud,
};
use lemmy_utils::error::LemmyError;
use lemmy_websocket::{
  send::{
    send_comment_ws_message_simple,
    send_community_ws_message,
    send_pm_ws_message,
    send_post_ws_message,
  },
  LemmyContext,
  UserOperationCrud,
};
use std::ops::Deref;
use url::Url;

pub mod delete;
pub mod delete_user;
pub mod undo_delete;

/// Parameter `reason` being set indicates that this is a removal by a mod. If its unset, this
/// action was done by a normal user.
#[tracing::instrument(skip_all)]
pub async fn send_apub_delete_in_community(
  actor: Person,
  community: Community,
  object: DeletableObjects,
  reason: Option<String>,
  deleted: bool,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let actor = ApubPerson::from(actor);
  let activity = if deleted {
    let delete = Delete::new(&actor, object, public(), Some(&community), reason, context)?;
    AnnouncableActivities::Delete(delete)
  } else {
    let undo = UndoDelete::new(&actor, object, public(), Some(&community), reason, context)?;
    AnnouncableActivities::UndoDelete(undo)
  };
  send_activity_in_community(activity, &actor, &community.into(), vec![], context).await
}

#[tracing::instrument(skip_all)]
pub async fn send_apub_delete_private_message(
  actor: &ApubPerson,
  pm: PrivateMessage,
  deleted: bool,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let recipient_id = pm.recipient_id;
  let recipient: ApubPerson =
    blocking(context.pool(), move |conn| Person::read(conn, recipient_id))
      .await??
      .into();

  let deletable = DeletableObjects::PrivateMessage(Box::new(pm.into()));
  let inbox = vec![recipient.shared_inbox_or_inbox()];
  if deleted {
    let delete = Delete::new(actor, deletable, recipient.actor_id(), None, None, context)?;
    send_lemmy_activity(context, delete, actor, inbox, true).await?;
  } else {
    let undo = UndoDelete::new(actor, deletable, recipient.actor_id(), None, None, context)?;
    send_lemmy_activity(context, undo, actor, inbox, true).await?;
  };
  Ok(())
}

pub enum DeletableObjects {
  Community(Box<ApubCommunity>),
  Comment(Box<ApubComment>),
  Post(Box<ApubPost>),
  PrivateMessage(Box<ApubPrivateMessage>),
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
    if let Some(p) = ApubPrivateMessage::read_from_apub_id(ap_id.clone(), context).await? {
      return Ok(DeletableObjects::PrivateMessage(Box::new(p)));
    }
    Err(diesel::NotFound.into())
  }

  pub(crate) fn id(&self) -> Url {
    match self {
      DeletableObjects::Community(c) => c.actor_id(),
      DeletableObjects::Comment(c) => c.ap_id.clone().into(),
      DeletableObjects::Post(p) => p.ap_id.clone().into(),
      DeletableObjects::PrivateMessage(p) => p.ap_id.clone().into(),
    }
  }
}

#[tracing::instrument(skip_all)]
pub(in crate::activities) async fn verify_delete_activity(
  activity: &Delete,
  is_mod_action: bool,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let object = DeletableObjects::read_from_db(activity.object.id(), context).await?;
  match object {
    DeletableObjects::Community(community) => {
      verify_is_public(&activity.to, &[])?;
      if community.local {
        // can only do this check for local community, in remote case it would try to fetch the
        // deleted community (which fails)
        verify_person_in_community(&activity.actor, &community, context, request_counter).await?;
      }
      // community deletion is always a mod (or admin) action
      verify_mod_action(
        &activity.actor,
        activity.object.id(),
        &community,
        context,
        request_counter,
      )
      .await?;
    }
    DeletableObjects::Post(p) => {
      verify_is_public(&activity.to, &[])?;
      verify_delete_post_or_comment(
        &activity.actor,
        &p.ap_id.clone().into(),
        &activity.get_community(context, request_counter).await?,
        is_mod_action,
        context,
        request_counter,
      )
      .await?;
    }
    DeletableObjects::Comment(c) => {
      verify_is_public(&activity.to, &[])?;
      verify_delete_post_or_comment(
        &activity.actor,
        &c.ap_id.clone().into(),
        &activity.get_community(context, request_counter).await?,
        is_mod_action,
        context,
        request_counter,
      )
      .await?;
    }
    DeletableObjects::PrivateMessage(_) => {
      verify_person(&activity.actor, context, request_counter).await?;
      verify_domains_match(activity.actor.inner(), activity.object.id())?;
    }
  }
  Ok(())
}

#[tracing::instrument(skip_all)]
async fn verify_delete_post_or_comment(
  actor: &ObjectId<ApubPerson>,
  object_id: &Url,
  community: &ApubCommunity,
  is_mod_action: bool,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  verify_person_in_community(actor, community, context, request_counter).await?;
  if is_mod_action {
    verify_mod_action(actor, object_id, community, context, request_counter).await?;
  } else {
    // domain of post ap_id and post.creator ap_id are identical, so we just check the former
    verify_domains_match(actor.inner(), object_id)?;
  }
  Ok(())
}

/// Write deletion or restoring of an object to the database, and send websocket message.
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
        let mod_: Person = actor
          .dereference(context, local_instance(context), request_counter)
          .await?
          .deref()
          .clone();
        let object = DeletableObjects::Community(community.clone());
        let c: Community = community.deref().deref().clone();
        send_apub_delete_in_community(mod_, c, object, None, true, context).await?;
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
    DeletableObjects::PrivateMessage(pm) => {
      let deleted_private_message = blocking(context.pool(), move |conn| {
        PrivateMessage::update_deleted(conn, pm.id, deleted)
      })
      .await??;

      send_pm_ws_message(
        deleted_private_message.id,
        UserOperationCrud::DeletePrivateMessage,
        None,
        context,
      )
      .await?;
    }
  }
  Ok(())
}
