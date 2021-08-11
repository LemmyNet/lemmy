use crate::{
  activities::{verify_mod_action, verify_person_in_community},
  ActorType,
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{verify_domains_match, ActivityCommonFields};
use lemmy_db_queries::ApubObject;
use lemmy_db_schema::{
  source::{comment::Comment, community::Community, post::Post},
  DbUrl,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

pub mod delete;
pub mod undo_delete;

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

pub(in crate::activities::deletion) async fn verify_delete_activity(
  object: &Url,
  cc: &Url,
  common: &ActivityCommonFields,
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
      verify_mod_action(&common.actor, c.actor_id(), context).await?;
    }
    DeletableObjects::Post(p) => {
      verify_person_in_community(&common.actor, cc, context, request_counter).await?;
      // domain of post ap_id and post.creator ap_id are identical, so we just check the former
      verify_domains_match(&common.actor, &p.ap_id.into())?;
    }
    DeletableObjects::Comment(c) => {
      verify_person_in_community(&common.actor, cc, context, request_counter).await?;
      verify_domains_match(&common.actor, &c.ap_id.into())?;
    }
  }
  Ok(())
}
