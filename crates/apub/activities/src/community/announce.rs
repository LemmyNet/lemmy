use crate::{
  activity_lists::{
    AnnouncableActivities,
    AnnouncableActivitiesWrapper::{self, Many, Single},
  },
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

// NOTE: This is no longer needed because the wrapper enum handles it
// impl Id for RawAnnouncableActivities {
//   fn id(&self) -> &Url {
//     &self.id
//   }
// }

#[async_trait::async_trait]
impl Collection for AnnouncableActivitiesCollection {
  #[doc = " Actor or object that this collection belongs to"]
  type Owner = ApubCommunity;

  #[doc = " App data type passed to handlers. Must be identical to"]
  #[doc = " [crate::config::FederationConfigBuilder::app_data] type."]
  type DataType = LemmyContext;

  #[doc = " The type of protocol struct which gets sent over network to federate this database struct."]
  type Kind = AnnouncableActivitiesCollection;

  #[doc = " Error type returned by handler methods"]
  type Error = LemmyError;

  #[doc = " Reads local collection from database and returns it as Activitypub JSON."]
  #[expect(
    mismatched_lifetime_syntaxes,
    clippy::type_complexity,
    clippy::type_repetition_in_bounds
  )]
  async fn read_local(
    _owner: &Self::Owner,
    _context: &Data<Self::DataType>,
  ) -> Result<Self::Kind, Self::Error> {
    todo!()
  }

  #[doc = " Verifies that the received object is valid."]
  #[doc = ""]
  #[doc = " You should check here that the domain of id matches `expected_domain`. Additionally you"]
  #[doc = " should perform any application specific checks."]
  #[expect(
    mismatched_lifetime_syntaxes,
    clippy::type_complexity,
    clippy::type_repetition_in_bounds
  )]
  async fn verify(
    json: &Self::Kind,
    expected_domain: &Url,
    _context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    verify_domains_match(expected_domain, &json.id.clone())?;
    Ok(())
  }

  #[doc = " Convert object from ActivityPub type to database type."]
  #[doc = ""]
  #[doc = " Called when an object is received from HTTP fetch or as part of an activity. This method"]
  #[doc = " should also write the received object to database. Note that there is no distinction"]
  #[doc = " between create and update, so an `upsert` operation should be used."]
  #[expect(
    mismatched_lifetime_syntaxes,
    clippy::type_complexity,
    clippy::type_repetition_in_bounds
  )]
  async fn from_json(
    _json: Self::Kind,
    _owner: &Self::Owner,
    _context: &Data<LemmyContext>,
  ) -> LemmyResult<Self> {
    todo!()
  }
}

impl Id for OneOrManyActivity {
  fn id(&self) -> &Url {
    match self {
      OneOrManyActivity::One(a) => &a.id,
      OneOrManyActivity::Many(ac) => &ac.id,
    }
  }
}

// NOTE: not needed because the nested object in AnnounceActivity is a collection, not an activity
// type #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct InnerActivities {
//   id: Url,
//   total_activities: i32,
//   activities: Vec<AnnouncableActivities>,
// }
// #[async_trait::async_trait]
// impl Activity for InnerActivities {
//   type DataType = LemmyContext;
//   type Error = LemmyError;
//
//   fn id(&self) -> &Url {
//     &self.id
//   }
//
//   fn actor(&self) -> &Url {
//     unimplemented!()
//   }
//
//   async fn verify(&self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
//     Ok(())
//   }
//
//   async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
//     for activity in self.activities {
//       activity.receive(context).await?
//     }
//     Ok(())
//   }
// }

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
    let wrapper: OneOrManyActivity = self.object.dereference(context).await?;
    if let Ok(Single(object)) = wrapper.try_into() {
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
    } else if let Many(object_collection) = wrapper.try_into()? {
      todo!()
    }
    // todo!()
  }
}

// This won't be needed anymore I think
impl TryFrom<RawAnnouncableActivities> for AnnouncableActivities {
  type Error = serde_json::error::Error;

  fn try_from(value: RawAnnouncableActivities) -> Result<Self, Self::Error> {
    let mut map = value.other.clone();
    map.insert("id".to_string(), Value::String(value.id.to_string()));
    map.insert("actor".to_string(), Value::String(value.actor.to_string()));
    serde_json::from_value(Value::Object(map))
  }
}

impl TryFrom<OneOrManyActivity> for AnnouncableActivitiesWrapper {
  type Error = serde_json::error::Error;

  fn try_from(value: OneOrManyActivity) -> Result<Self, Self::Error> {
    match value {
      OneOrManyActivity::One(a) => {
        let mut map = a.other.clone();
        map.insert("id".to_string(), Value::String(a.id.to_string()));
        map.insert("actor".to_string(), Value::String(a.actor.to_string()));
        serde_json::from_value(Value::Object(map))
      }
      OneOrManyActivity::Many(ac) => {
        let mut activities = vec![];
        for a in ac.ordered_items {
          let mut map = a.other.clone();
          map.insert("id".to_string(), Value::String(a.id.to_string()));
          map.insert("actor".to_string(), Value::String(a.actor.to_string()));
          activities.push(Value::Object(map));
        }
        serde_json::from_value(Value::Array(activities))
      }
    }
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
