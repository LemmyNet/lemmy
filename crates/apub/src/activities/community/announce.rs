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

#[async_trait::async_trait(?Send)]
pub(crate) trait GetCommunity {
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError>;
}

impl AnnounceActivity {
  fn new(
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

  pub async fn send(
    object: AnnouncableActivities,
    community: &ApubCommunity,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let announce = AnnounceActivity::new(object.clone(), community, context)?;
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

    // Pleroma (and likely Mastodon) can't handle activities like Announce/Create/Page. So for
    // compatibility to allow them to follow Lemmy communities, we also send Announce/Page and
    // Announce/Note (for new and updated posts/comments).
    use AnnouncableActivities::*;
    let object = match object {
      CreateOrUpdatePost(c) => Page(c.object),
      CreateOrUpdateComment(c) => Note(c.object),
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
    let object_value = serde_json::to_value(&self.object)?;
    let object_data: ActivityCommonFields = serde_json::from_value(object_value.to_owned())?;

    if is_activity_already_known(context.pool(), &object_data.id).await? {
      return Ok(());
    }
    insert_activity(&object_data.id, object_value, false, true, context.pool()).await?;
    self.object.receive(context, request_counter).await
  }
}
