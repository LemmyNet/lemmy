use crate::{
  activities::{
    comment::{
      create::CreateComment,
      delete::DeleteComment,
      dislike::DislikeComment,
      like::LikeComment,
      remove::RemoveComment,
      undo_delete::UndoDeleteComment,
      undo_dislike::UndoDislikeComment,
      undo_like::UndoLikeComment,
      undo_remove::UndoRemoveComment,
      update::UpdateComment,
    },
    community::{
      block_user::BlockUserFromCommunity,
      delete::DeleteCommunity,
      remove::RemoveCommunity,
      undo_block_user::UndoBlockUserFromCommunity,
      undo_delete::UndoDeleteCommunity,
      undo_remove::UndoRemoveCommunity,
      update::UpdateCommunity,
    },
    post::{
      create::CreatePost,
      delete::DeletePost,
      dislike::DislikePost,
      like::LikePost,
      remove::RemovePost,
      undo_delete::UndoDeletePost,
      undo_dislike::UndoDislikePost,
      undo_like::UndoLikePost,
      undo_remove::UndoRemovePost,
      update::UpdatePost,
    },
  },
  inbox::{is_activity_already_known, new_inbox_routing::Activity},
};
use activitystreams::activity::kind::RemoveType;
use lemmy_apub::check_is_apub_id_valid;
use lemmy_apub_lib::{verify_domains_match, PublicUrl, ReceiveActivity, VerifyActivity};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum AnnouncableActivities {
  CreateComment(CreateComment),
  UpdateComment(UpdateComment),
  LikeComment(LikeComment),
  DislikeComment(DislikeComment),
  UndoLikeComment(UndoLikeComment),
  UndoDislikeComment(UndoDislikeComment),
  DeleteComment(DeleteComment),
  UndoDeleteComment(UndoDeleteComment),
  RemoveComment(RemoveComment),
  UndoRemoveComment(UndoRemoveComment),
  CreatePost(CreatePost),
  UpdatePost(UpdatePost),
  LikePost(LikePost),
  DislikePost(DislikePost),
  DeletePost(DeletePost),
  UndoDeletePost(UndoDeletePost),
  RemovePost(RemovePost),
  UndoRemovePost(UndoRemovePost),
  UndoLikePost(UndoLikePost),
  UndoDislikePost(UndoDislikePost),
  // TODO: which of these get announced?
  UpdateCommunity(UpdateCommunity),
  DeleteCommunity(DeleteCommunity),
  RemoveCommunity(RemoveCommunity),
  UndoDeleteCommunity(UndoDeleteCommunity),
  UndoRemoveCommunity(UndoRemoveCommunity),
  BlockUserFromCommunity(BlockUserFromCommunity),
  UndoBlockUserFromCommunity(UndoBlockUserFromCommunity),
}

#[async_trait::async_trait(?Send)]
impl VerifyActivity for AnnouncableActivities {
  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    self.verify(context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for AnnouncableActivities {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    self.receive(context, request_counter).await
  }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnounceActivity {
  to: PublicUrl,
  object: Activity<AnnouncableActivities>,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: RemoveType,
}

#[async_trait::async_trait(?Send)]
impl VerifyActivity for Activity<AnnounceActivity> {
  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.actor, self.id_unchecked())?;
    verify_domains_match(&self.actor, &self.inner.cc[0])?;
    check_is_apub_id_valid(&self.actor, false)?;
    self.inner.object.inner.verify(context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<AnnounceActivity> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    if is_activity_already_known(context.pool(), &self.inner.object.id_unchecked()).await? {
      return Ok(());
    }
    self
      .inner
      .object
      .inner
      .receive(context, request_counter)
      .await
  }
}
