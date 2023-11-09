use crate::{
  activities::{
    generate_activity_id,
    generate_announce_activity_id,
    send_lemmy_activity,
    verify_is_public,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  insert_received_activity,
  objects::community::ApubCommunity,
  protocol::{
    activities::community::announce::{AnnounceActivity, RawAnnouncableActivities},
    Id,
    IdOrNestedObject,
    InCommunity,
  },
};
use activitypub_federation::{
  config::Data,
  kinds::{activity::AnnounceType, public},
  traits::{ActivityHandler, Actor},
};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::{activity::ActivitySendTargets, community::CommunityFollower};
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
use serde_json::Value;
use url::Url;

#[async_trait::async_trait]
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
  async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
    let activity: AnnouncableActivities = self.clone().try_into()?;

    // This is only for sending, not receiving so we reject it.
    if let AnnouncableActivities::Page(_) = activity {
      Err(LemmyErrorType::CannotReceivePage)?
    }

    // Need to treat community as optional here because `Delete/PrivateMessage` gets routed through
    let community = activity.community(context).await.ok();
    can_accept_activity_in_community(&community, context).await?;

    // verify and receive activity
    activity.verify(context).await?;
    activity.clone().receive(context).await?;

    // if community is local, send activity to followers
    if let Some(community) = community {
      if community.local {
        let actor_id = activity.actor().clone().into();
        verify_person_in_community(&actor_id, &community, context).await?;
        AnnounceActivity::send(self, &community, context).await?;
      }
    }
    Ok(())
  }
}

impl AnnounceActivity {
  pub(crate) fn new(
    object: RawAnnouncableActivities,
    community: &ApubCommunity,
    context: &Data<LemmyContext>,
  ) -> Result<AnnounceActivity, LemmyError> {
    let inner_kind = object
      .other
      .get("type")
      .and_then(serde_json::Value::as_str)
      .unwrap_or("other");
    let id =
      generate_announce_activity_id(inner_kind, &context.settings().get_protocol_and_hostname())?;
    Ok(AnnounceActivity {
      actor: community.id().into(),
      to: vec![public()],
      object: IdOrNestedObject::NestedObject(object),
      cc: vec![community.followers_url.clone().into()],
      kind: AnnounceType::Announce,
      id,
    })
  }

  #[tracing::instrument(skip_all)]
  pub async fn send(
    object: RawAnnouncableActivities,
    community: &ApubCommunity,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let announce = AnnounceActivity::new(object.clone(), community, context)?;
    let inboxes = ActivitySendTargets::to_local_community_followers(community.id);
    send_lemmy_activity(context, announce, community, inboxes.clone(), false).await?;

    // Pleroma and Mastodon can't handle activities like Announce/Create/Page. So for
    // compatibility, we also send Announce/Page so that they can follow Lemmy communities.
    let object_parsed = object.try_into()?;
    if let AnnouncableActivities::CreateOrUpdatePost(c) = object_parsed {
      // Hack: need to convert Page into a format which can be sent as activity, which requires
      //       adding actor field.
      let announcable_page = RawAnnouncableActivities {
        id: generate_activity_id(
          AnnounceType::Announce,
          &context.settings().get_protocol_and_hostname(),
        )?,
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

#[async_trait::async_trait]
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
  async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), LemmyError> {
    insert_received_activity(&self.id, context).await?;
    verify_is_public(&self.to, &self.cc)?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<Self::DataType>) -> Result<(), LemmyError> {
    let object: AnnouncableActivities = self.object.object(context).await?.try_into()?;

    // This is only for sending, not receiving so we reject it.
    if let AnnouncableActivities::Page(_) = object {
      Err(LemmyErrorType::CannotReceivePage)?
    }

    let community = object.community(context).await?;
    can_accept_activity_in_community(&Some(community), context).await?;

    // verify here in order to avoid fetching the object twice over http
    object.verify(context).await?;
    object.receive(context).await
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

/// Check if an activity in the given community can be accepted. To return true, the community must
/// either be local to this instance, or it must have at least one local follower.
///
/// TODO: This means mentions dont work if the community has no local followers. Can be fixed
///       by checking if any local user is in to/cc fields of activity. Anyway this is a minor
///       problem compared to receiving unsolicited posts.
async fn can_accept_activity_in_community(
  community: &Option<ApubCommunity>,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  if let Some(community) = community {
    if !community.local
      && !CommunityFollower::has_local_followers(&mut context.pool(), community.id).await?
    {
      Err(LemmyErrorType::CommunityHasNoFollowers)?
    }
    // Local only community can't federate
    if community.local_only {
      Err(LemmyErrorType::CouldntFindCommunity)
    }
  }
  Ok(())
}
