use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_is_public},
  activity_lists::AnnouncableActivities,
  insert_activity,
  objects::community::ApubCommunity,
  protocol::{
    activities::{community::announce::AnnounceActivity, CreateOrUpdateType},
    IdOrNestedObject,
  },
  ActorType,
};
use activitypub_federation::{core::object_id::ObjectId, data::Data, traits::ActivityHandler};
use activitystreams_kinds::{activity::AnnounceType, public};
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use tracing::debug;
use url::Url;

#[async_trait::async_trait(?Send)]
pub(crate) trait GetCommunity {
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError>;
}

impl AnnounceActivity {
  pub(crate) fn new(
    object: AnnouncableActivities,
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
      unparsed: Default::default(),
    })
  }

  #[tracing::instrument(skip_all)]
  pub async fn send(
    object: AnnouncableActivities,
    community: &ApubCommunity,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let announce = AnnounceActivity::new(object.clone(), community, context)?;
    let inboxes = community.get_follower_inboxes(context).await?;
    send_lemmy_activity(context, announce, community, inboxes.clone(), false).await?;

    // Pleroma and Mastodon can't handle activities like Announce/Create/Page. So for
    // compatibility, we also send Announce/Page so that they can follow Lemmy communities.
    use AnnouncableActivities::*;
    let object = match object {
      CreateOrUpdatePost(c) if c.kind == CreateOrUpdateType::Create => Page(c.object),
      _ => return Ok(()),
    };
    let announce_compat = AnnounceActivity::new(object, community, context)?;
    send_lemmy_activity(context, announce_compat, community, inboxes, false).await?;
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
    let object = self.object.object(context, request_counter).await?;
    // we have to verify this here in order to avoid fetching the object twice over http
    object.verify(context, request_counter).await?;

    // TODO: this can probably be implemented in a cleaner way
    match object {
      // Dont insert these into activities table, as they are not activities.
      AnnouncableActivities::Page(_) => {}
      _ => {
        let object_value = serde_json::to_value(&object)?;
        let insert =
          insert_activity(object.id(), object_value, false, true, context.pool()).await?;
        if !insert {
          debug!(
            "Received duplicate activity in announce {}",
            object.id().to_string()
          );
          return Ok(());
        }
      }
    }
    object.receive(context, request_counter).await
  }
}
