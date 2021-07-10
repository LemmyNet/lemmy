use activitystreams::activity::kind::AnnounceType;
use lemmy_apub_lib::{ActivityCommonFields, ActivityHandlerNew, PublicUrl};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
  activities::{
    comment::{
      create::CreateComment,
      delete::DeleteComment,
      remove::RemoveComment,
      undo_delete::UndoDeleteComment,
      undo_remove::UndoRemoveComment,
      update::UpdateComment,
    },
    community::{block_user::BlockUserFromCommunity, undo_block_user::UndoBlockUserFromCommunity},
    post::{
      create::CreatePost,
      delete::DeletePost,
      remove::RemovePost,
      undo_delete::UndoDeletePost,
      undo_remove::UndoRemovePost,
      update::UpdatePost,
    },
    post_or_comment::{
      dislike::DislikePostOrComment,
      like::LikePostOrComment,
      undo_dislike::UndoDislikePostOrComment,
      undo_like::UndoLikePostOrComment,
    },
    verify_activity,
    verify_community,
  },
  http::is_activity_already_known,
};

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandlerNew)]
#[serde(untagged)]
pub enum AnnouncableActivities {
  CreateComment(CreateComment),
  UpdateComment(UpdateComment),
  DeleteComment(DeleteComment),
  UndoDeleteComment(UndoDeleteComment),
  RemoveComment(RemoveComment),
  UndoRemoveComment(UndoRemoveComment),
  CreatePost(CreatePost),
  UpdatePost(UpdatePost),
  DeletePost(DeletePost),
  UndoDeletePost(UndoDeletePost),
  RemovePost(RemovePost),
  UndoRemovePost(UndoRemovePost),
  LikePostOrComment(LikePostOrComment),
  DislikePostOrComment(DislikePostOrComment),
  UndoLikePostOrComment(UndoLikePostOrComment),
  UndoDislikePostOrComment(UndoDislikePostOrComment),
  BlockUserFromCommunity(BlockUserFromCommunity),
  UndoBlockUserFromCommunity(UndoBlockUserFromCommunity),
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnounceActivity {
  to: PublicUrl,
  object: AnnouncableActivities,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: AnnounceType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandlerNew for AnnounceActivity {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    verify_community(&self.common.actor, context, request_counter).await?;
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
    self.object.receive(context, request_counter).await
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
