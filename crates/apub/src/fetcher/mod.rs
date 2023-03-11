use activitypub_federation::{
  config::RequestData,
  fetch::{object_id::ObjectId, webfinger::webfinger_resolve_actor},
  traits::{Actor, ApubObject},
};
use itertools::Itertools;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::traits::ApubActor;
use lemmy_utils::error::LemmyError;

pub mod post_or_comment;
pub mod search;
pub mod user_or_community;

/// Resolve actor identifier (eg `!news@example.com`) from local database to avoid network requests.
/// This only works for local actors, and remote actors which were previously fetched (so it doesnt
/// trigger any new fetch).
#[tracing::instrument(skip_all)]
pub async fn resolve_actor_identifier<ActorType, DbActor>(
  identifier: &str,
  context: &RequestData<LemmyContext>,
  include_deleted: bool,
) -> Result<DbActor, LemmyError>
where
  ActorType:
    ApubObject<DataType = LemmyContext, Error = LemmyError> + ApubObject + Actor + Send + 'static,
  for<'de2> <ActorType as ApubObject>::ApubType: serde::Deserialize<'de2>,
  DbActor: ApubActor + Send + 'static,
{
  // remote actor
  if identifier.contains('@') {
    let (name, domain) = identifier
      .splitn(2, '@')
      .collect_tuple()
      .expect("invalid query");
    let name = name.to_string();
    let domain = format!("{}://{}", context.settings().get_protocol_string(), domain);
    let actor = DbActor::read_from_name_and_domain(context.pool(), &name, &domain).await;
    if actor.is_ok() {
      Ok(actor?)
    } else {
      // Fetch the actor from its home instance using webfinger
      let actor: ActorType = webfinger_resolve_actor(identifier, context).await?;
      Ok(actor)
    }
  }
  // local actor
  else {
    let identifier = identifier.to_string();
    Ok(DbActor::read_from_name(context.pool(), &identifier, include_deleted).await?)
  }
}
