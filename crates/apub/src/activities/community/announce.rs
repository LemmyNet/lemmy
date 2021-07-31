use crate::{
  activities::{
    comment::create_or_update::CreateOrUpdateComment,
    community::{
      add_mod::AddMod,
      block_user::BlockUserFromCommunity,
      list_community_follower_inboxes,
      undo_block_user::UndoBlockUserFromCommunity,
    },
    deletion::{
      delete::DeletePostCommentOrCommunity,
      undo_delete::UndoDeletePostCommentOrCommunity,
    },
    generate_activity_id,
    post::create_or_update::CreateOrUpdatePost,
    removal::{
      remove::RemovePostCommentCommunityOrMod,
      undo_remove::UndoRemovePostCommentOrCommunity,
    },
    verify_activity,
    verify_community,
    voting::{
      dislike::DislikePostOrComment,
      like::LikePostOrComment,
      undo_dislike::UndoDislikePostOrComment,
      undo_like::UndoLikePostOrComment,
    },
  },
  activity_queue::send_activity_new,
  extensions::context::lemmy_context,
  http::is_activity_already_known,
  insert_activity,
  ActorType,
  CommunityType,
};
use activitystreams::activity::kind::AnnounceType;
use lemmy_apub_lib::{values::PublicUrl, ActivityCommonFields, ActivityHandler};
use lemmy_db_schema::source::community::Community;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler)]
#[serde(untagged)]
pub enum AnnouncableActivities {
  CreateOrUpdateComment(CreateOrUpdateComment),
  CreateOrUpdatePost(Box<CreateOrUpdatePost>),
  LikePostOrComment(LikePostOrComment),
  DislikePostOrComment(DislikePostOrComment),
  UndoLikePostOrComment(UndoLikePostOrComment),
  UndoDislikePostOrComment(UndoDislikePostOrComment),
  DeletePostCommentOrCommunity(DeletePostCommentOrCommunity),
  UndoDeletePostCommentOrCommunity(UndoDeletePostCommentOrCommunity),
  RemovePostCommentCommunityOrMod(RemovePostCommentCommunityOrMod),
  UndoRemovePostCommentOrCommunity(UndoRemovePostCommentOrCommunity),
  BlockUserFromCommunity(BlockUserFromCommunity),
  UndoBlockUserFromCommunity(UndoBlockUserFromCommunity),
  AddMod(AddMod),
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnounceActivity {
  to: PublicUrl,
  object: AnnouncableActivities,
  cc: Vec<Url>,
  #[serde(rename = "type")]
  kind: AnnounceType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

impl AnnounceActivity {
  pub async fn send(
    object: AnnouncableActivities,
    community: &Community,
    additional_inboxes: Vec<Url>,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let announce = AnnounceActivity {
      to: PublicUrl::Public,
      object,
      cc: vec![community.followers_url()],
      kind: AnnounceType::Announce,
      common: ActivityCommonFields {
        context: lemmy_context(),
        id: generate_activity_id(&AnnounceType::Announce)?,
        actor: community.actor_id(),
        unparsed: Default::default(),
      },
    };
    let inboxes = list_community_follower_inboxes(community, additional_inboxes, context).await?;
    send_activity_new(
      context,
      &announce,
      &announce.common.id,
      community,
      inboxes,
      false,
    )
    .await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for AnnounceActivity {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    verify_community(&self.common.actor, context, request_counter).await?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    if is_activity_already_known(context.pool(), self.object.common().id_unchecked()).await? {
      return Ok(());
    }
    insert_activity(
      self.object.common().id_unchecked(),
      self.object.clone(),
      false,
      true,
      context.pool(),
    )
    .await?;
    self.object.receive(context, request_counter).await
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
