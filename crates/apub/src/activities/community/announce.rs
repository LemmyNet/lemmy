use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_activity, verify_is_public},
  activity_lists::AnnouncableActivities,
  http::{is_activity_already_known, ActivityCommonFields},
  insert_activity,
  objects::community::ApubCommunity,
  protocol::activities::community::announce::AnnounceActivity,
};
use activitystreams::{activity::kind::AnnounceType, public};
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  traits::{ActivityHandler, ActorType},
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
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
  pub async fn send(
    object: AnnouncableActivities,
    community: &ApubCommunity,
    additional_inboxes: Vec<Url>,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let announce = AnnounceActivity {
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
    };
    let inboxes = community
      .get_follower_inboxes(additional_inboxes, context)
      .await?;
    send_lemmy_activity(context, &announce, &announce.id, community, inboxes, false).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for AnnounceActivity {
  type DataType = LemmyContext;
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

  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    // TODO: this is pretty ugly, but i cant think of a much better way
    let object = serde_json::to_string(&self.object)?;
    let object_data: ActivityCommonFields = serde_json::from_str(&object)?;

    if is_activity_already_known(context, &object_data.id).await? {
      return Ok(());
    }
    insert_activity(
      &object_data.id,
      self.object.clone(),
      false,
      true,
      context.pool(),
    )
    .await?;
    self.object.receive(context, request_counter).await
  }
}
