use crate::{
  apub::{
    check_is_apub_id_valid,
    community::do_announce,
    extensions::signatures::verify,
    fetcher::{
      get_or_fetch_and_upsert_actor,
      get_or_fetch_and_upsert_community,
      get_or_fetch_and_upsert_user,
    },
    inbox::activities::{
      announce::receive_announce,
      create::receive_create,
      delete::receive_delete,
      dislike::receive_dislike,
      like::receive_like,
      remove::receive_remove,
      undo::receive_undo,
      update::receive_update,
    },
    insert_activity,
  },
  routes::{ChatServerParam, DbPoolParam},
  DbPool,
  LemmyError,
};
use activitystreams::{
  activity::{ActorAndObject, ActorAndObjectRef},
  base::{AsBase, Extends},
  object::AsObject,
  prelude::*,
};
use actix_web::{client::Client, web, HttpRequest, HttpResponse};
use lemmy_db::user::User_;
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use url::Url;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum ValidTypes {
  Create,
  Update,
  Like,
  Dislike,
  Delete,
  Undo,
  Remove,
  Announce,
}

// TODO: this isnt entirely correct, cause some of these activities are not ActorAndObject,
//       but it might still work due to the anybase conversion
pub type AcceptedActivities = ActorAndObject<ValidTypes>;

/// Handler for all incoming activities to user inboxes.
pub async fn shared_inbox(
  request: HttpRequest,
  input: web::Json<AcceptedActivities>,
  client: web::Data<Client>,
  pool: DbPoolParam,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let activity = input.into_inner();

  let json = serde_json::to_string(&activity)?;
  debug!("Shared inbox received activity: {}", json);

  let sender = &activity.actor()?.to_owned().single_xsd_any_uri().unwrap();
  let community = get_community_id_from_activity(&activity)?;

  check_is_apub_id_valid(sender)?;
  check_is_apub_id_valid(&community)?;

  let actor = get_or_fetch_and_upsert_actor(sender, &client, &pool).await?;
  verify(&request, actor.as_ref())?;

  let any_base = activity.clone().into_any_base()?;
  let kind = activity.kind().unwrap();
  let res = match kind {
    ValidTypes::Announce => receive_announce(any_base, &client, &pool, chat_server).await,
    ValidTypes::Create => receive_create(any_base, &client, &pool, chat_server).await,
    ValidTypes::Update => receive_update(any_base, &client, &pool, chat_server).await,
    ValidTypes::Like => receive_like(any_base, &client, &pool, chat_server).await,
    ValidTypes::Dislike => receive_dislike(any_base, &client, &pool, chat_server).await,
    ValidTypes::Remove => receive_remove(any_base, &client, &pool, chat_server).await,
    ValidTypes::Delete => receive_delete(any_base, &client, &pool, chat_server).await,
    ValidTypes::Undo => receive_undo(any_base, &client, &pool, chat_server).await,
  };

  insert_activity(actor.user_id(), activity.clone(), false, &pool).await?;
  res
}

pub(in crate::apub::inbox) fn receive_unhandled_activity<A>(
  activity: A,
) -> Result<HttpResponse, LemmyError>
where
  A: Debug,
{
  debug!("received unhandled activity type: {:?}", activity);
  Ok(HttpResponse::NotImplemented().finish())
}

pub(in crate::apub::inbox) async fn get_user_from_activity<T, A>(
  activity: &T,
  client: &Client,
  pool: &DbPool,
) -> Result<User_, LemmyError>
where
  T: AsBase<A> + ActorAndObjectRef,
{
  let actor = activity.actor()?;
  let user_uri = actor.as_single_xsd_any_uri().unwrap();
  get_or_fetch_and_upsert_user(&user_uri, client, pool).await
}

pub(in crate::apub::inbox) fn get_community_id_from_activity<T, A>(
  activity: &T,
) -> Result<Url, LemmyError>
where
  T: AsBase<A> + ActorAndObjectRef + AsObject<A>,
{
  let cc = activity.cc().unwrap();
  let cc = cc.as_many().unwrap();
  Ok(cc.first().unwrap().as_xsd_any_uri().unwrap().to_owned())
}

pub(in crate::apub::inbox) async fn announce_if_community_is_local<T, Kind>(
  activity: T,
  user: &User_,
  client: &Client,
  pool: &DbPool,
) -> Result<(), LemmyError>
where
  T: AsObject<Kind>,
  T: Extends<Kind>,
  Kind: Serialize,
  <T as Extends<Kind>>::Error: From<serde_json::Error> + Send + Sync + 'static,
{
  let cc = activity.cc().unwrap();
  let cc = cc.as_many().unwrap();
  let community_followers_uri = cc.first().unwrap().as_xsd_any_uri().unwrap();
  // TODO: this is hacky but seems to be the only way to get the community ID
  let community_uri = community_followers_uri
    .to_string()
    .replace("/followers", "");
  let community =
    get_or_fetch_and_upsert_community(&Url::parse(&community_uri)?, client, pool).await?;

  if community.local {
    do_announce(activity.into_any_base()?, &community, &user, client, pool).await?;
  }
  Ok(())
}
