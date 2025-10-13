use crate::{
  activity_lists::AnnouncableActivities,
  generate_activity_id,
  generate_announce_activity_id,
  protocol::{
    community::announce::{AnnounceActivity, RawAnnouncableActivities},
    IdOrNestedObject,
  },
  send_lemmy_activity,
};
use activitypub_federation::{
  config::Data,
  kinds::activity::AnnounceType,
  traits::{Activity, Object},
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::community::ApubCommunity,
  utils::{
    functions::{generate_to, verify_person_in_community, verify_visibility},
    protocol::{Id, InCommunity},
  },
};
use lemmy_db_schema::source::{activity::ActivitySendTargets, community::CommunityActions};
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult, UntranslatedError};
use serde_json::Value;
use url::Url;

#[async_trait::async_trait]
impl Activity for RawAnnouncableActivities {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    &self.actor
  }

  async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
    let activity: AnnouncableActivities = self.clone().try_into()?;

    // This is only for sending, not receiving so we reject it.
    if let AnnouncableActivities::Page(_) = activity {
      Err(UntranslatedError::CannotReceivePage)?
    }

    // Need to treat community as optional here because `Delete/PrivateMessage` gets routed through
    let community = activity.community(context).await.ok();
    can_accept_activity_in_community(&community, context).await?;

    // verify and receive activity
    activity.verify(context).await?;
    let ap_id = activity.actor().clone().into();
    activity.receive(context).await?;

    // if community is local, send activity to followers
    if let Some(community) = community {
      if community.local {
        verify_person_in_community(&ap_id, &community, context).await?;
        AnnounceActivity::send(self, &community, context).await?;
      }
    }

    Ok(())
  }
}

impl Id for RawAnnouncableActivities {
  fn id(&self) -> &Url {
    &self.id
  }
}

impl AnnounceActivity {
  pub fn new(
    object: RawAnnouncableActivities,
    community: &ApubCommunity,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<AnnounceActivity> {
    let inner_kind = object
      .other
      .get("type")
      .and_then(serde_json::Value::as_str)
      .unwrap_or("other");
    let id =
      generate_announce_activity_id(inner_kind, &context.settings().get_protocol_and_hostname())?;
    Ok(AnnounceActivity {
      actor: community.id().clone().into(),
      to: generate_to(community)?,
      object: IdOrNestedObject::NestedObject(object),
      cc: community
        .followers_url
        .clone()
        .map(Into::into)
        .into_iter()
        .collect(),
      kind: AnnounceType::Announce,
      id,
    })
  }

  pub async fn send(
    object: RawAnnouncableActivities,
    community: &ApubCommunity,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
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
        id: generate_activity_id(AnnounceType::Announce, context)?,
        actor: c.actor.clone().into_inner(),
        other: serde_json::to_value(c.object)?
          .as_object()
          .ok_or(UntranslatedError::Unreachable)?
          .clone(),
      };
      let announce_compat = AnnounceActivity::new(announcable_page, community, context)?;
      send_lemmy_activity(context, announce_compat, community, inboxes, false).await?;
    }
    Ok(())
  }
}

#[async_trait::async_trait]
impl Activity for AnnounceActivity {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let object: AnnouncableActivities = self.object.object(context).await?.try_into()?;

    // This is only for sending, not receiving so we reject it.
    if let AnnouncableActivities::Page(_) = object {
      Err(UntranslatedError::CannotReceivePage)?
    }

    let community = object.community(context).await?;
    verify_visibility(&self.to, &self.cc, &community)?;
    can_accept_activity_in_community(&Some(community), context).await?;

    // verify here in order to avoid fetching the object twice over http
    object.verify(context).await?;
    object.receive(context).await
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
    // Local only community can't federate
    if !community.visibility.can_federate() {
      return Err(LemmyErrorType::NotFound.into());
    }
    if !community.local {
      CommunityActions::check_accept_activity_in_community(&mut context.pool(), community.id)
        .await?
    }
  }
  Ok(())
}
