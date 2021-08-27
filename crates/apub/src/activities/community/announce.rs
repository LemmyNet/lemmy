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
    undo_remove::UndoRemovePostCommentOrCommunity,
    verify_activity,
    verify_community,
    voting::{undo_vote::UndoVote, vote::Vote},
  },
  activity_queue::send_activity_new,
  extensions::context::lemmy_context,
  http::is_activity_already_known,
  insert_activity,
  ActorType,
  CommunityType,
};
use activitystreams::{
  activity::kind::AnnounceType,
  base::AnyBase,
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use lemmy_apub_lib::{values::PublicUrl, ActivityFields, ActivityHandler};
use lemmy_db_schema::source::community::Community;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler, ActivityFields)]
#[serde(untagged)]
pub enum AnnouncableActivities {
  CreateOrUpdateComment(CreateOrUpdateComment),
  CreateOrUpdatePost(Box<CreateOrUpdatePost>),
  Vote(Vote),
  UndoVote(UndoVote),
  Delete(Delete),
  UndoDelete(UndoDelete),
  UndoRemovePostCommentOrCommunity(UndoRemovePostCommentOrCommunity),
  UpdateCommunity(Box<UpdateCommunity>),
  BlockUserFromCommunity(BlockUserFromCommunity),
  UndoBlockUserFromCommunity(UndoBlockUserFromCommunity),
  AddMod(AddMod),
  RemoveMod(RemoveMod),
}

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct AnnounceActivity {
  actor: Url,
  to: [PublicUrl; 1],
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
    community: &Community,
    additional_inboxes: Vec<Url>,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let announce = AnnounceActivity {
      actor: community.actor_id(),
      to: [PublicUrl::Public],
      object,
      cc: vec![community.followers_url()],
      kind: AnnounceType::Announce,
      id: generate_activity_id(&AnnounceType::Announce)?,
      context: lemmy_context(),
      unparsed: Default::default(),
    };
    let inboxes = list_community_follower_inboxes(community, additional_inboxes, context).await?;
    send_activity_new(context, &announce, &announce.id, community, inboxes, false).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for AnnounceActivity {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self)?;
    verify_community(&self.actor, context, request_counter).await?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
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
