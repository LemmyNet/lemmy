use crate::federation::ApubPerson;
use activitypub_federation::{
  config::Data,
  fetch::webfinger::webfinger_resolve_actor,
  traits::{Actor, Object},
};
use diesel::NotFound;
use itertools::Itertools;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::objects::{community::ApubCommunity, multi_community::ApubMultiCommunity};
use lemmy_db_schema::{
  newtypes::{CommunityId, MultiCommunityId},
  source::{community::Community, multi_community::MultiCommunity, person::Person},
  traits::ApubActor,
};
use lemmy_db_schema_file::PersonId;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};

/// Resolve actor identifier like `!news@example.com` to user or community object.
///
/// In case the requesting user is logged in and the object was not found locally, it is attempted
/// to fetch via webfinger from the original instance.
async fn resolve_ap_identifier<ActorType, DbActor>(
  identifier: &str,
  context: &Data<LemmyContext>,
  local_user_view: &Option<LocalUserView>,
  include_deleted: bool,
) -> LemmyResult<ActorType>
where
  ActorType: Object<DataType = LemmyContext, Error = LemmyError>
    + Object
    + Actor
    + From<DbActor>
    + Send
    + Sync
    + 'static,
  for<'de2> <ActorType as Object>::Kind: serde::Deserialize<'de2>,
  DbActor: ApubActor + Send + 'static,
{
  // remote actor
  if identifier.contains('@') {
    let (name, domain) = identifier
      .splitn(2, '@')
      .collect_tuple()
      .ok_or(LemmyErrorType::InvalidUrl)?;
    let actor = DbActor::read_from_name(&mut context.pool(), name, Some(domain), false)
      .await
      .ok()
      .flatten();
    if let Some(actor) = actor {
      Ok(actor.into())
    } else if local_user_view.is_some() {
      // Fetch the actor from its home instance using webfinger
      let actor: ActorType = webfinger_resolve_actor(&identifier.to_lowercase(), context).await?;
      Ok(actor)
    } else {
      Err(NotFound.into())
    }
  }
  // local actor
  else {
    let identifier = identifier.to_string();
    Ok(
      DbActor::read_from_name(&mut context.pool(), &identifier, None, include_deleted)
        .await?
        .ok_or(NotFound)?
        .into(),
    )
  }
}

pub(crate) async fn resolve_community_identifier(
  name: &Option<String>,
  id: Option<CommunityId>,
  context: &Data<LemmyContext>,
  local_user_view: &Option<LocalUserView>,
) -> LemmyResult<Option<CommunityId>> {
  Ok(if let Some(name) = name {
    Some(
      resolve_ap_identifier::<ApubCommunity, Community>(name, context, local_user_view, true)
        .await?
        .id,
    )
  } else {
    id
  })
}

pub(crate) async fn resolve_person_identifier(
  id: Option<PersonId>,
  username: &Option<String>,
  context: &Data<LemmyContext>,
  local_user_view: &Option<LocalUserView>,
) -> LemmyResult<PersonId> {
  Ok(
    if let Some(name) = username {
      Some(
        resolve_ap_identifier::<ApubPerson, Person>(name, context, local_user_view, true)
          .await?
          .id,
      )
    } else {
      id
    }
    .ok_or(LemmyErrorType::NoIdGiven)?,
  )
}

pub(crate) async fn resolve_multi_community_identifier(
  name: &Option<String>,
  id: Option<MultiCommunityId>,
  context: &Data<LemmyContext>,
  local_user_view: &Option<LocalUserView>,
) -> LemmyResult<Option<MultiCommunityId>> {
  Ok(if let Some(name) = name {
    Some(
      resolve_ap_identifier::<ApubMultiCommunity, MultiCommunity>(
        name,
        context,
        local_user_view,
        true,
      )
      .await?
      .id,
    )
  } else {
    id
  })
}
