use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_activity, verify_is_public},
  activity_lists::AnnouncableActivities,
  http::ActivityCommonFields,
  insert_activity,
  objects::community::ApubCommunity,
  protocol::activities::{community::announce::AnnounceActivity, CreateOrUpdateType},
};
use activitystreams_kinds::{activity::AnnounceType, public};
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  traits::{ActivityHandler, ActorType},
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use tracing::info;

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
      object,
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
    // temporary hack to get activity id of object
    let object_fields: ActivityCommonFields =
      serde_json::from_value(serde_json::to_value(&object)?)?;
    info!(
      "Announcing activity {} as {}",
      object_fields.id, announce.id
    );

    let inboxes = community.get_follower_inboxes(context).await?;
    send_lemmy_activity(
      context,
      &announce,
      &announce.id,
      community,
      inboxes.clone(),
      false,
    )
    .await?;

    // Pleroma and Mastodon can't handle activities like Announce/Create/Page. So for
    // compatibility, we also send Announce/Page so that they can follow Lemmy communities.
    use AnnouncableActivities::*;
    let object = match object {
      CreateOrUpdatePost(c) if c.kind == CreateOrUpdateType::Create => Page(c.object),
      _ => return Ok(()),
    };
    let announce_compat = AnnounceActivity::new(object, community, context)?;
    send_lemmy_activity(
      context,
      &announce_compat,
      &announce_compat.id,
      community,
      inboxes,
      false,
    )
    .await?;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for AnnounceActivity {
  type DataType = LemmyContext;

  #[tracing::instrument(skip_all)]
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to, &self.cc)?;
    verify_activity(&self.id, self.actor.inner(), &context.settings())?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    // TODO: this can probably be implemented in a cleaner way
    match self.object {
      // Dont insert these into activities table, as they are not activities.
      AnnouncableActivities::Page(_) => {}
      _ => {
        let object_value = serde_json::to_value(&self.object)?;
        let object_data: ActivityCommonFields = serde_json::from_value(object_value.to_owned())?;

        insert_activity(&object_data.id, object_value, false, true, context.pool()).await?;
      }
    }
    self.object.receive(context, request_counter).await
  }
}
