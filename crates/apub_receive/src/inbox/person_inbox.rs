use crate::{
  activities::receive::{
    comment::{receive_create_comment, receive_update_comment},
    community::{
      receive_delete_community,
      receive_remove_community,
      receive_undo_delete_community,
      receive_undo_remove_community,
    },
    private_message::{
      receive_create_private_message,
      receive_delete_private_message,
      receive_undo_delete_private_message,
      receive_update_private_message,
    },
    receive_unhandled_activity,
    verify_activity_domains_valid,
  },
  inbox::{
    assert_activity_not_local,
    get_activity_id,
    inbox_verify_http_signature,
    is_activity_already_known,
    is_addressed_to_community_followers,
    is_addressed_to_local_person,
    receive_for_community::{
      receive_add_for_community,
      receive_create_for_community,
      receive_delete_for_community,
      receive_dislike_for_community,
      receive_like_for_community,
      receive_remove_for_community,
      receive_undo_for_community,
      receive_update_for_community,
    },
    verify_is_addressed_to_public,
  },
};
use activitystreams::{
  activity::{Accept, ActorAndObject, Announce, Create, Delete, Follow, Remove, Undo, Update},
  base::AnyBase,
  prelude::*,
};
use actix_web::{web, HttpRequest, HttpResponse};
use anyhow::{anyhow, Context};
use diesel::NotFound;
use lemmy_api_common::blocking;
use lemmy_apub::{
  check_is_apub_id_valid,
  fetcher::community::get_or_fetch_and_upsert_community,
  get_activity_to_and_cc,
  insert_activity,
  ActorType,
};
use lemmy_db_queries::{source::person::Person_, ApubObject, Followable};
use lemmy_db_schema::source::{
  community::{Community, CommunityFollower},
  person::Person,
  private_message::PrivateMessage,
};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use strum_macros::EnumString;
use url::Url;

/// Allowed activities for person inbox.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum PersonValidTypes {
  Accept,   // community accepted our follow request
  Create,   // create private message
  Update,   // edit private message
  Delete,   // private message or community deleted by creator
  Undo,     // private message or community restored
  Remove,   // community removed by admin
  Announce, // post, comment or vote in community
}

pub type PersonAcceptedActivities = ActorAndObject<PersonValidTypes>;

/// Handler for all incoming activities to person inboxes.
pub async fn person_inbox(
  request: HttpRequest,
  input: web::Json<PersonAcceptedActivities>,
  path: web::Path<String>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let activity = input.into_inner();
  // First of all check the http signature
  let request_counter = &mut 0;
  let actor = inbox_verify_http_signature(&activity, &context, request, request_counter).await?;

  // Do nothing if we received the same activity before
  let activity_id = get_activity_id(&activity, &actor.actor_id())?;
  if is_activity_already_known(context.pool(), &activity_id).await? {
    return Ok(HttpResponse::Ok().finish());
  }

  // Check if the activity is actually meant for us
  let username = path.into_inner();
  let person = blocking(&context.pool(), move |conn| {
    Person::find_by_name(&conn, &username)
  })
  .await??;
  let to_and_cc = get_activity_to_and_cc(&activity);
  // TODO: we should also accept activities that are sent to community followers
  if !to_and_cc.contains(&&person.actor_id()) {
    return Err(anyhow!("Activity delivered to wrong person").into());
  }

  assert_activity_not_local(&activity)?;
  insert_activity(&activity_id, activity.clone(), false, true, context.pool()).await?;

  debug!(
    "Person {} received activity {:?} from {}",
    person.name,
    &activity.id_unchecked(),
    &actor.actor_id()
  );

  person_receive_message(
    activity.clone(),
    Some(person.clone()),
    actor.as_ref(),
    &context,
    request_counter,
  )
  .await
}

/// Receives Accept/Follow, Announce, private messages and community (undo) remove, (undo) delete
pub(crate) async fn person_receive_message(
  activity: PersonAcceptedActivities,
  to_person: Option<Person>,
  actor: &dyn ActorType,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<HttpResponse, LemmyError> {
  is_for_person_inbox(context, &activity).await?;

  let any_base = activity.clone().into_any_base()?;
  let kind = activity.kind().context(location_info!())?;
  let actor_url = actor.actor_id();
  match kind {
    PersonValidTypes::Accept => {
      receive_accept(
        &context,
        any_base,
        actor,
        to_person.expect("person provided"),
        request_counter,
      )
      .await?;
    }
    PersonValidTypes::Announce => {
      Box::pin(receive_announce(&context, any_base, actor, request_counter)).await?
    }
    PersonValidTypes::Create => {
      Box::pin(receive_create(
        &context,
        any_base,
        actor_url,
        request_counter,
      ))
      .await?
    }
    PersonValidTypes::Update => {
      Box::pin(receive_update(
        &context,
        any_base,
        actor_url,
        request_counter,
      ))
      .await?
    }
    PersonValidTypes::Delete => {
      Box::pin(receive_delete(
        context,
        any_base,
        &actor_url,
        request_counter,
      ))
      .await?
    }
    PersonValidTypes::Undo => {
      Box::pin(receive_undo(context, any_base, &actor_url, request_counter)).await?
    }
    PersonValidTypes::Remove => Box::pin(receive_remove(context, any_base, &actor_url)).await?,
  };

  // TODO: would be logical to move websocket notification code here

  Ok(HttpResponse::Ok().finish())
}

/// Returns true if the activity is addressed directly to one or more local persons, or if it is
/// addressed to the followers collection of a remote community, and at least one local person follows
/// it.
async fn is_for_person_inbox(
  context: &LemmyContext,
  activity: &PersonAcceptedActivities,
) -> Result<(), LemmyError> {
  let to_and_cc = get_activity_to_and_cc(activity);
  // Check if it is addressed directly to any local person
  if is_addressed_to_local_person(&to_and_cc, context.pool()).await? {
    return Ok(());
  }

  // Check if it is addressed to any followers collection of a remote community, and that the
  // community has local followers.
  let community = is_addressed_to_community_followers(&to_and_cc, context.pool()).await?;
  if let Some(c) = community {
    let community_id = c.id;
    let has_local_followers = blocking(&context.pool(), move |conn| {
      CommunityFollower::has_local_followers(conn, community_id)
    })
    .await??;
    if c.local {
      return Err(
        anyhow!("Remote activity cant be addressed to followers of local community").into(),
      );
    }
    if has_local_followers {
      return Ok(());
    }
  }

  Err(anyhow!("Not addressed for any local person").into())
}

/// Handle accepted follows.
async fn receive_accept(
  context: &LemmyContext,
  activity: AnyBase,
  actor: &dyn ActorType,
  person: Person,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let accept = Accept::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&accept, &actor.actor_id(), false)?;

  let object = accept.object().to_owned().one().context(location_info!())?;
  let follow = Follow::from_any_base(object)?.context(location_info!())?;
  verify_activity_domains_valid(&follow, &person.actor_id(), false)?;

  let community_uri = accept
    .actor()?
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;

  let community =
    get_or_fetch_and_upsert_community(&community_uri, context, request_counter).await?;

  let community_id = community.id;
  let person_id = person.id;
  // This will throw an error if no follow was requested
  blocking(&context.pool(), move |conn| {
    CommunityFollower::follow_accepted(conn, community_id, person_id)
  })
  .await??;

  Ok(())
}

#[derive(EnumString)]
enum AnnouncableActivities {
  Create,
  Update,
  Like,
  Dislike,
  Delete,
  Remove,
  Undo,
  Add,
}

/// Takes an announce and passes the inner activity to the appropriate handler.
pub async fn receive_announce(
  context: &LemmyContext,
  activity: AnyBase,
  actor: &dyn ActorType,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let announce = Announce::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&announce, &actor.actor_id(), false)?;
  verify_is_addressed_to_public(&announce)?;

  let kind = announce
    .object()
    .as_single_kind_str()
    .and_then(|s| s.parse().ok());
  let inner_activity = announce
    .object()
    .to_owned()
    .one()
    .context(location_info!())?;

  let inner_id = inner_activity.id().context(location_info!())?.to_owned();
  check_is_apub_id_valid(&inner_id)?;
  if is_activity_already_known(context.pool(), &inner_id).await? {
    return Ok(());
  }

  use AnnouncableActivities::*;
  match kind {
    Some(Create) => {
      receive_create_for_community(context, inner_activity, &inner_id, request_counter).await
    }
    Some(Update) => {
      receive_update_for_community(
        context,
        inner_activity,
        Some(announce),
        &inner_id,
        request_counter,
      )
      .await
    }
    Some(Like) => {
      receive_like_for_community(context, inner_activity, &inner_id, request_counter).await
    }
    Some(Dislike) => {
      receive_dislike_for_community(context, inner_activity, &inner_id, request_counter).await
    }
    Some(Delete) => {
      receive_delete_for_community(
        context,
        inner_activity,
        Some(announce),
        &inner_id,
        request_counter,
      )
      .await
    }
    Some(Remove) => {
      receive_remove_for_community(context, inner_activity, Some(announce), request_counter).await
    }
    Some(Undo) => {
      receive_undo_for_community(
        context,
        inner_activity,
        Some(announce),
        &inner_id,
        request_counter,
      )
      .await
    }
    Some(Add) => {
      receive_add_for_community(context, inner_activity, Some(announce), request_counter).await
    }
    _ => receive_unhandled_activity(inner_activity),
  }
}

/// Receive either a new private message, or a new comment mention. We distinguish them by checking
/// whether the activity is public.
async fn receive_create(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let create = Create::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&create, &expected_domain, true)?;
  if verify_is_addressed_to_public(&create).is_ok() {
    receive_create_comment(create, context, request_counter).await
  } else {
    receive_create_private_message(&context, create, expected_domain, request_counter).await
  }
}

/// Receive either an updated private message, or an updated comment mention. We distinguish
/// them by checking whether the activity is public.
async fn receive_update(
  context: &LemmyContext,
  activity: AnyBase,
  expected_domain: Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let update = Update::from_any_base(activity)?.context(location_info!())?;
  verify_activity_domains_valid(&update, &expected_domain, true)?;
  if verify_is_addressed_to_public(&update).is_ok() {
    receive_update_comment(update, context, request_counter).await
  } else {
    receive_update_private_message(&context, update, expected_domain, request_counter).await
  }
}

async fn receive_delete(
  context: &LemmyContext,
  any_base: AnyBase,
  expected_domain: &Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  use CommunityOrPrivateMessage::*;

  let delete = Delete::from_any_base(any_base.clone())?.context(location_info!())?;
  verify_activity_domains_valid(&delete, expected_domain, true)?;
  let object_uri = delete
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;

  match find_community_or_private_message_by_id(context, object_uri).await? {
    Community(c) => receive_delete_community(context, c).await,
    PrivateMessage(p) => receive_delete_private_message(context, delete, p, request_counter).await,
  }
}

async fn receive_remove(
  context: &LemmyContext,
  any_base: AnyBase,
  expected_domain: &Url,
) -> Result<(), LemmyError> {
  let remove = Remove::from_any_base(any_base.clone())?.context(location_info!())?;
  verify_activity_domains_valid(&remove, expected_domain, true)?;
  let object_uri = remove
    .object()
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_apub_id(conn, &object_uri.into())
  })
  .await??;
  receive_remove_community(&context, community).await
}

async fn receive_undo(
  context: &LemmyContext,
  any_base: AnyBase,
  expected_domain: &Url,
  request_counter: &mut i32,
) -> Result<(), LemmyError> {
  let undo = Undo::from_any_base(any_base)?.context(location_info!())?;
  verify_activity_domains_valid(&undo, expected_domain, true)?;

  let inner_activity = undo.object().to_owned().one().context(location_info!())?;
  let kind = inner_activity.kind_str();
  match kind {
    Some("Delete") => {
      let delete = Delete::from_any_base(inner_activity)?.context(location_info!())?;
      verify_activity_domains_valid(&delete, expected_domain, true)?;
      let object_uri = delete
        .object()
        .to_owned()
        .single_xsd_any_uri()
        .context(location_info!())?;
      use CommunityOrPrivateMessage::*;
      match find_community_or_private_message_by_id(context, object_uri).await? {
        Community(c) => receive_undo_delete_community(context, c).await,
        PrivateMessage(p) => {
          receive_undo_delete_private_message(context, undo, expected_domain, p, request_counter)
            .await
        }
      }
    }
    Some("Remove") => {
      let remove = Remove::from_any_base(inner_activity)?.context(location_info!())?;
      let object_uri = remove
        .object()
        .to_owned()
        .single_xsd_any_uri()
        .context(location_info!())?;
      let community = blocking(context.pool(), move |conn| {
        Community::read_from_apub_id(conn, &object_uri.into())
      })
      .await??;
      receive_undo_remove_community(context, community).await
    }
    _ => receive_unhandled_activity(undo),
  }
}
enum CommunityOrPrivateMessage {
  Community(Community),
  PrivateMessage(PrivateMessage),
}

async fn find_community_or_private_message_by_id(
  context: &LemmyContext,
  apub_id: Url,
) -> Result<CommunityOrPrivateMessage, LemmyError> {
  let ap_id = apub_id.to_owned();
  let community = blocking(context.pool(), move |conn| {
    Community::read_from_apub_id(conn, &ap_id.into())
  })
  .await?;
  if let Ok(c) = community {
    return Ok(CommunityOrPrivateMessage::Community(c));
  }

  let ap_id = apub_id.to_owned();
  let private_message = blocking(context.pool(), move |conn| {
    PrivateMessage::read_from_apub_id(conn, &ap_id.into())
  })
  .await?;
  if let Ok(p) = private_message {
    return Ok(CommunityOrPrivateMessage::PrivateMessage(p));
  }

  Err(NotFound.into())
}
