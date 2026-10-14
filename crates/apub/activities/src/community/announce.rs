use crate::{
  activity_lists::AnnouncableActivities,
  generate_activity_id,
  generate_announce_activity_id,
  protocol::{
    IdOrNestedObject,
    community::announce::{
      AnnouncableActivitiesCollection,
      AnnounceActivity,
      OneOrManyActivity,
      RawAnnouncableActivities,
    },
  },
  send_lemmy_activity,
};
use activitypub_federation::{
  config::Data,
  kinds::activity::AnnounceType,
  protocol::verification::{verify_domains_match, verify_urls_match},
  traits::{Activity, Collection, Object},
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
use tracing::warn;
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
      return Err(UntranslatedError::CannotReceivePage.into());
    }

    // Need to treat community as optional here because `Delete/PrivateMessage` gets routed through
    let community = activity.community(context).await.ok();
    can_accept_activity_in_community(&community, context).await?;

    // verify and receive activity
    activity.verify(context).await?;
    let ap_id = activity.actor().clone().into();
    activity.receive(context).await?;

    // if community is local, send activity to followers
    if let Some(community) = community
      && community.local
    {
      verify_person_in_community(&ap_id, &community, context).await?;
      AnnounceActivity::send(self, &community, context).await?;
    }

    Ok(())
  }
}

/// Marker type for receiving a batch of activities (FEP-1a11), where `Announce.object` is an
/// `OrderedCollection` of activities.
#[derive(Clone, Debug)]
pub struct ApubAnnouncedBatch(());

#[async_trait::async_trait]
impl Collection for ApubAnnouncedBatch {
  type Owner = ApubCommunity;
  type DataType = LemmyContext;
  type Kind = AnnouncableActivitiesCollection;
  type Error = LemmyError;

  async fn read_local(
    _owner: &Self::Owner,
    _context: &Data<Self::DataType>,
  ) -> LemmyResult<Self::Kind> {
    // Unsupported for now. Could later serve batches persisted for crash recovery (see from_json).
    Err(LemmyErrorType::NotFound.into())
  }

  async fn verify(
    json: &Self::Kind,
    expected_domain: &Url,
    _context: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    verify_domains_match(expected_domain, &json.id)?;
    Ok(())
  }

  async fn from_json(
    _json: Self::Kind,
    _owner: &Self::Owner,
    _context: &Data<Self::DataType>,
  ) -> LemmyResult<Self> {
    // Unsupported for now. Could later persist received batches temporarily, so unprocessed items
    // can be recovered after a crash.
    Err(LemmyErrorType::NotFound.into())
  }
}

impl Id for OneOrManyActivity {
  fn id(&self) -> &Url {
    match self {
      OneOrManyActivity::One(a) => &a.id,
      OneOrManyActivity::Many(c) => &c.id,
    }
  }
}

impl AnnounceActivity {
  pub fn new(
    object: RawAnnouncableActivities,
    community: &ApubCommunity,
    object_id: Option<&Url>,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<AnnounceActivity> {
    let inner_kind = object
      .other
      .get("type")
      .and_then(serde_json::Value::as_str)
      .unwrap_or("other");
    let id = generate_announce_activity_id(
      inner_kind,
      &context.settings().get_protocol_and_hostname(),
      object_id,
    )?;
    Ok(AnnounceActivity {
      actor: community.id().clone().into(),
      to: generate_to(community)?,
      object: IdOrNestedObject::NestedObject(OneOrManyActivity::One(object)),
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
    let announce = AnnounceActivity::new(object.clone(), community, None, context)?;
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
      let announce_compat = AnnounceActivity::new(announcable_page, community, None, context)?;
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
    match self.object.dereference(context).await? {
      OneOrManyActivity::One(raw) => {
        let object: AnnouncableActivities = raw.try_into()?;

        // This is only for sending, not receiving so we reject it.
        if let AnnouncableActivities::Page(_) = object {
          return Err(UntranslatedError::CannotReceivePage.into());
        }

        let community = object.community(context).await?;
        verify_urls_match(community.ap_id.inner(), self.actor.inner())?;
        verify_visibility(&self.to, &self.cc, &community)?;
        can_accept_activity_in_community(&Some(community), context).await?;

        // verify here in order to avoid fetching the object twice over http
        object.verify(context).await?;
        object.receive(context).await
      }
      OneOrManyActivity::Many(batch) => {
        let community = self.actor.dereference(context).await?;
        verify_visibility(&self.to, &self.cc, &community)?;
        can_accept_activity_in_community(&Some(community.clone()), context).await?;
        ApubAnnouncedBatch::verify(&batch, self.actor.inner(), context).await?;
        receive_batch(batch, &community, context).await
      }
    }
  }
}

/// Receives each activity of a batch in order. A failing activity is logged and skipped, so that it
/// doesn't prevent the rest of the batch from being received.
async fn receive_batch(
  batch: AnnouncableActivitiesCollection,
  community: &ApubCommunity,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  for raw in batch.ordered_items {
    let id = raw.id.clone();
    if let Err(e) = receive_batch_item(raw, community, context).await {
      warn!("Skipping activity {id} in batch {}: {e}", batch.id);
    }
  }
  Ok(())
}

async fn receive_batch_item(
  raw: RawAnnouncableActivities,
  community: &ApubCommunity,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let activity: AnnouncableActivities = raw.try_into()?;
  if !can_batch(&activity) {
    return Err(LemmyErrorType::NotFound.into());
  }
  let activity_community = activity.community(context).await?;
  verify_urls_match(activity_community.ap_id.inner(), community.ap_id.inner())?;
  activity.verify(context).await?;
  activity.receive(context).await
}

/// Activity types which are accepted as part of a batch. Initially only votes, as in FEP-1a11.
fn can_batch(activity: &AnnouncableActivities) -> bool {
  matches!(
    activity,
    AnnouncableActivities::Vote(_) | AnnouncableActivities::UndoVote(_)
  )
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
      CommunityActions::check_accept_activity_in_community(&mut context.pool(), community).await?
    }
  }
  Ok(())
}
