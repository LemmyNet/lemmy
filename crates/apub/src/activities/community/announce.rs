use crate::{
  activities::{
    comment::create_or_update::CreateOrUpdateComment,
    community::{
      add_mod::AddMod,
      block_user::BlockUserFromCommunity,
      list_community_follower_inboxes,
      remove_mod::RemoveMod,
      undo_block_user::UndoBlockUserFromCommunity,
      update::UpdateCommunity,
    },
    deletion::{delete::Delete, undo_delete::UndoDelete},
    generate_activity_id,
    post::create_or_update::CreateOrUpdatePost,
    verify_activity,
    verify_is_public,
    voting::{undo_vote::UndoVote, vote::Vote},
  },
  context::lemmy_context,
  fetcher::object_id::ObjectId,
  http::is_activity_already_known,
  insert_activity,
  objects::community::ApubCommunity,
  send_lemmy_activity,
};
use activitystreams::{
  activity::kind::AnnounceType,
  base::AnyBase,
  primitives::OneOrMany,
  public,
  unparsed::Unparsed,
};
use lemmy_apub_lib::{
  data::Data,
  traits::{ActivityFields, ActivityHandler, ActorType},
  verify::verify_urls_match,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler, ActivityFields)]
#[serde(untagged)]
#[activity_handler(LemmyContext)]
pub enum AnnouncableActivities {
  CreateOrUpdateComment(CreateOrUpdateComment),
  CreateOrUpdatePost(Box<CreateOrUpdatePost>),
  Vote(Vote),
  UndoVote(UndoVote),
  Delete(Delete),
  UndoDelete(UndoDelete),
  UpdateCommunity(Box<UpdateCommunity>),
  BlockUserFromCommunity(BlockUserFromCommunity),
  UndoBlockUserFromCommunity(UndoBlockUserFromCommunity),
  AddMod(AddMod),
  RemoveMod(RemoveMod),
}

#[async_trait::async_trait(?Send)]
pub(crate) trait GetCommunity {
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError>;
}

#[async_trait::async_trait(?Send)]
impl GetCommunity for AnnouncableActivities {
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    use AnnouncableActivities::*;
    let community = match self {
      CreateOrUpdateComment(a) => a.get_community(context, request_counter).await?,
      CreateOrUpdatePost(a) => a.get_community(context, request_counter).await?,
      Vote(a) => a.get_community(context, request_counter).await?,
      UndoVote(a) => a.get_community(context, request_counter).await?,
      Delete(a) => a.get_community(context, request_counter).await?,
      UndoDelete(a) => a.get_community(context, request_counter).await?,
      UpdateCommunity(a) => a.get_community(context, request_counter).await?,
      BlockUserFromCommunity(a) => a.get_community(context, request_counter).await?,
      UndoBlockUserFromCommunity(a) => a.get_community(context, request_counter).await?,
      AddMod(a) => a.get_community(context, request_counter).await?,
      RemoveMod(a) => a.get_community(context, request_counter).await?,
    };
    verify_urls_match(self.actor(), &community.actor_id())?;
    Ok(community)
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct AnnounceActivity {
  actor: ObjectId<ApubCommunity>,
  to: Vec<Url>,
  object: AnnouncableActivities,
  cc: Vec<Url>,
  #[serde(rename = "type")]
  kind: AnnounceType,
  id: Url,
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(flatten)]
  unparsed: Unparsed,
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
      cc: vec![community.followers_url.clone().into_inner()],
      kind: AnnounceType::Announce,
      id: generate_activity_id(
        &AnnounceType::Announce,
        &context.settings().get_protocol_and_hostname(),
      )?,
      context: lemmy_context(),
      unparsed: Default::default(),
    };
    let inboxes = list_community_follower_inboxes(community, additional_inboxes, context).await?;
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
    verify_is_public(&self.to)?;
    verify_activity(self, &context.settings())?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    if is_activity_already_known(context.pool(), self.object.id_unchecked()).await? {
      return Ok(());
    }
    insert_activity(
      self.object.id_unchecked(),
      self.object.clone(),
      false,
      true,
      context.pool(),
    )
    .await?;
    self.object.receive(context, request_counter).await
  }
}
