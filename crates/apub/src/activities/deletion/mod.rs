use crate::{
  activities::{
    deletion::{delete::Delete, undo_delete::UndoDelete},
    verify_mod_action,
    verify_person_in_community,
  },
  ActorType,
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{verify_domains_match, ActivityCommonFields};
use lemmy_db_queries::ApubObject;
use lemmy_db_schema::{
  source::{comment::Comment, community::Community, person::Person, post::Post},
  DbUrl,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

pub mod delete;
pub mod undo_delete;

pub async fn send_apub_delete(
  actor: &Person,
  community: &Community,
  object_id: Url,
  deleted: bool,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  if deleted {
    Delete::send(actor, community, object_id, None, context).await
  } else {
    UndoDelete::send(actor, community, object_id, None, context).await
  }
}

// TODO: remove reason is actually optional in lemmy. we set an empty string in that case, but its
//       ugly
pub async fn send_apub_remove(
  actor: &Person,
  community: &Community,
  object_id: Url,
  reason: String,
  removed: bool,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  if removed {
    Delete::send(actor, community, object_id, Some(reason), context).await
  } else {
    UndoDelete::send(actor, community, object_id, Some(reason), context).await
  }
}

pub enum DeletableObjects {
  Community(Box<Community>),
  Comment(Box<Comment>),
  Post(Box<Post>),
}

impl DeletableObjects {
  pub(crate) async fn read_from_db(
    ap_id: &Url,
    context: &LemmyContext,
  ) -> Result<DeletableObjects, LemmyError> {
    let id: DbUrl = ap_id.clone().into();

    if let Some(c) = DeletableObjects::read_type_from_db::<Community>(id.clone(), context).await? {
      return Ok(DeletableObjects::Community(Box::new(c)));
    }
    if let Some(p) = DeletableObjects::read_type_from_db::<Post>(id.clone(), context).await? {
      return Ok(DeletableObjects::Post(Box::new(p)));
    }
    if let Some(c) = DeletableObjects::read_type_from_db::<Comment>(id.clone(), context).await? {
      return Ok(DeletableObjects::Comment(Box::new(c)));
    }
    Err(diesel::NotFound.into())
  }

  // TODO: a method like this should be provided by fetcher module
  async fn read_type_from_db<Type: ApubObject + Send + 'static>(
    ap_id: DbUrl,
    context: &LemmyContext,
  ) -> Result<Option<Type>, LemmyError> {
    blocking(context.pool(), move |conn| {
      Type::read_from_apub_id(conn, &ap_id).ok()
    })
    .await
  }
}

pub(in crate::activities) async fn verify_delete_activity(
  object: &Url,
  cc: &Url,
  common: &ActivityCommonFields,
  is_mod_action: bool,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let object = DeletableObjects::read_from_db(object, context).await?;
  match object {
    DeletableObjects::Community(c) => {
      if c.local {
        // can only do this check for local community, in remote case it would try to fetch the
        // deleted community (which fails)
        verify_person_in_community(&common.actor, cc, context, request_counter).await?;
      }
      // community deletion is always a mod (or admin) action
      verify_mod_action(&common.actor, c.actor_id(), context).await?;
    }
    DeletableObjects::Post(p) => {
      verify_delete_activity_post_or_comment(
        cc,
        common,
        &p.ap_id.into(),
        is_mod_action,
        context,
        request_counter,
      )
      .await?;
    }
    DeletableObjects::Comment(c) => {
      verify_delete_activity_post_or_comment(
        cc,
        common,
        &c.ap_id.into(),
        is_mod_action,
        context,
        request_counter,
      )
      .await?;
    }
  }
  Ok(())
}

async fn verify_delete_activity_post_or_comment(
  cc: &Url,
  common: &ActivityCommonFields,
  object_id: &Url,
  is_mod_action: bool,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  verify_person_in_community(&common.actor, cc, context, request_counter).await?;
  if is_mod_action {
    verify_mod_action(&common.actor, cc.clone(), context).await?;
  } else {
    // domain of post ap_id and post.creator ap_id are identical, so we just check the former
    verify_domains_match(&common.actor, object_id)?;
  }
  Ok(())
}
