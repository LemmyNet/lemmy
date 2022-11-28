use crate::{
  activities::{
    generate_activity_id,
    send_lemmy_activity,
    verify_is_public,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  insert_activity,
  objects::community::ApubCommunity,
  protocol::{
    activities::community::announce::{AnnounceActivity, RawAnnouncableActivities},
    Id,
    IdOrNestedObject,
    InCommunity,
  },
  ActorType,
};
use activitypub_federation::{core::object_id::ObjectId, data::Data, traits::ActivityHandler};
use activitystreams_kinds::{activity::AnnounceType, public};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyError;
use serde_json::Value;
use tracing::debug;
use url::Url;

#[async_trait::async_trait(?Send)]
impl ActivityHandler for RawAnnouncableActivities {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    &self.actor
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    &self,
    _data: &Data<Self::DataType>,
    _request_counter: &mut i32,
  ) -> Result<(), Self::Error> {
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    data: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), Self::Error> {
    let activity: AnnouncableActivities = self.clone().try_into()?;
    // This is only for sending, not receiving so we reject it.
    if let AnnouncableActivities::Page(_) = activity {
      return Err(LemmyError::from_message("Cant receive page"));
    }
    let community = activity.community(data, &mut 0).await?;
    let actor_id = ObjectId::new(activity.actor().clone());

    // verify and receive activity
    activity.verify(data, request_counter).await?;
    activity.receive(data, request_counter).await?;

    // send to community followers
    if community.local {
      verify_person_in_community(&actor_id, &community, data, &mut 0).await?;
      AnnounceActivity::send(self, &community, data).await?;
    }
    Ok(())
  }
}

impl AnnounceActivity {
  pub(crate) fn new(
    object: RawAnnouncableActivities,
    community: &ApubCommunity,
    context: &LemmyContext,
  ) -> Result<AnnounceActivity, LemmyError> {
    Ok(AnnounceActivity {
      actor: ObjectId::new(community.actor_id()),
      to: vec![public()],
      object: IdOrNestedObject::NestedObject(object),
      cc: vec![community.followers_url.clone().into()],
      kind: AnnounceType::Announce,
      id: generate_activity_id(
        &AnnounceType::Announce,
        &context.settings().get_protocol_and_hostname(),
      )?,
    })
  }

  #[tracing::instrument(skip_all)]
  pub async fn send(
    object: RawAnnouncableActivities,
    community: &ApubCommunity,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let announce = AnnounceActivity::new(object.clone(), community, context)?;
    let inboxes = community.get_follower_inboxes(context).await?;
    send_lemmy_activity(context, announce, community, inboxes.clone(), false).await?;

    // Pleroma and Mastodon can't handle activities like Announce/Create/Page. So for
    // compatibility, we also send Announce/Page so that they can follow Lemmy communities.
    let object_parsed = object.try_into()?;
    if let AnnouncableActivities::CreateOrUpdatePost(c) = object_parsed {
      // Hack: need to convert Page into a format which can be sent as activity, which requires
      //       adding actor field.
      let announcable_page = RawAnnouncableActivities {
        id: c.object.id.clone().into_inner(),
        actor: c.actor.clone().into_inner(),
        other: serde_json::to_value(c.object)?
          .as_object()
          .expect("is object")
          .clone(),
      };
      let announce_compat = AnnounceActivity::new(announcable_page, community, context)?;
      send_lemmy_activity(context, announce_compat, community, inboxes, false).await?;
    }
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for AnnounceActivity {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    &self,
    _context: &Data<LemmyContext>,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to, &self.cc)?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let object: AnnouncableActivities = self
      .object
      .object(context, request_counter)
      .await?
      .try_into()?;
    // This is only for sending, not receiving so we reject it.
    if let AnnouncableActivities::Page(_) = object {
      return Err(LemmyError::from_message("Cant receive page"));
    }

    // we have to verify this here in order to avoid fetching the object twice over http
    object.verify(context, request_counter).await?;

    let object_value = serde_json::to_value(&object)?;
    let insert = insert_activity(object.id(), object_value, false, true, context.pool()).await?;
    if !insert {
      debug!(
        "Received duplicate activity in announce {}",
        object.id().to_string()
      );
      return Ok(());
    }
    object.receive(context, request_counter).await
  }
}

impl Id for RawAnnouncableActivities {
  fn object_id(&self) -> &Url {
    ActivityHandler::id(self)
  }
}

impl TryFrom<RawAnnouncableActivities> for AnnouncableActivities {
  type Error = serde_json::error::Error;

  fn try_from(value: RawAnnouncableActivities) -> Result<Self, Self::Error> {
    let mut map = value.other.clone();
    map.insert("id".to_string(), Value::String(value.id.to_string()));
    map.insert("actor".to_string(), Value::String(value.actor.to_string()));
    serde_json::from_value(Value::Object(map))
  }
}

impl TryFrom<AnnouncableActivities> for RawAnnouncableActivities {
  type Error = serde_json::error::Error;

  fn try_from(value: AnnouncableActivities) -> Result<Self, Self::Error> {
    serde_json::from_value(serde_json::to_value(value)?)
  }
}
