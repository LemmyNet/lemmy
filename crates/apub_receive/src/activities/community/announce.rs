use crate::{
  activities::{
    comment::{create::CreateComment, update::UpdateComment},
    community::{block_user::BlockUserFromCommunity, undo_block_user::UndoBlockUserFromCommunity},
    post::{create::CreatePost, update::UpdatePost},
    post_or_comment::{
      delete::DeletePostOrComment,
      dislike::DislikePostOrComment,
      like::LikePostOrComment,
      remove::RemovePostOrComment,
      undo_delete::UndoDeletePostOrComment,
      undo_dislike::UndoDislikePostOrComment,
      undo_like::UndoLikePostOrComment,
      undo_remove::UndoRemovePostOrComment,
    },
    verify_activity,
    verify_community,
  },
  http::is_activity_already_known,
};
use activitystreams::activity::kind::AnnounceType;
use lemmy_apub::insert_activity;
use lemmy_apub_lib::{ActivityCommonFields, ActivityHandlerNew, PublicUrl};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandlerNew)]
#[serde(untagged)]
pub enum AnnouncableActivities {
  CreateComment(CreateComment),
  UpdateComment(UpdateComment),
  CreatePost(CreatePost),
  UpdatePost(UpdatePost),
  LikePostOrComment(LikePostOrComment),
  DislikePostOrComment(DislikePostOrComment),
  UndoLikePostOrComment(UndoLikePostOrComment),
  UndoDislikePostOrComment(UndoDislikePostOrComment),
  DeletePostOrComment(DeletePostOrComment),
  RemovePostOrComment(RemovePostOrComment),
  UndoRemovePostOrComment(UndoRemovePostOrComment),
  UndoDeletePostOrComment(UndoDeletePostOrComment),
  BlockUserFromCommunity(BlockUserFromCommunity),
  UndoBlockUserFromCommunity(UndoBlockUserFromCommunity),
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

#[async_trait::async_trait(?Send)]
impl ActivityHandlerNew for AnnounceActivity {
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
