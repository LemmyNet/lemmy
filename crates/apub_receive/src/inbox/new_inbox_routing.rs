use crate::activities::{
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
    announce::AnnounceActivity,
    block_user::BlockUserFromCommunity,
    delete::DeleteCommunity,
    remove::RemoveCommunity,
    undo_block_user::UndoBlockUserFromCommunity,
    undo_delete::UndoDeleteCommunity,
    undo_remove::UndoRemoveCommunity,
    update::UpdateCommunity,
  },
  follow::{accept::AcceptFollowCommunity, follow::FollowCommunity, undo::UndoFollowCommunity},
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
  private_message::{
    create::CreatePrivateMessage,
    delete::DeletePrivateMessage,
    undo_delete::UndoDeletePrivateMessage,
    update::UpdatePrivateMessage,
  },
};
use activitystreams::{base::AnyBase, primitives::OneOrMany, unparsed::Unparsed};
use lemmy_apub_lib::{ReceiveActivity, VerifyActivity};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

// TODO: would be nice if we could move this to lemmy_apub_lib crate. doing that gives error:
//       "only traits defined in the current crate can be implemented for arbitrary types"
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity<Kind> {
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  id: Url,
  pub(crate) actor: Url,

  /// type-specific fields
  #[serde(flatten)]
  pub inner: Kind,

  // unparsed fields
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl<Kind> Activity<Kind> {
  pub fn id_unchecked(&self) -> &Url {
    &self.id
  }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum SharedInboxActivities {
  FollowCommunity(FollowCommunity),
  AcceptFollowCommunity(AcceptFollowCommunity),
  UndoFollowCommunity(UndoFollowCommunity),
  CreatePrivateMessage(CreatePrivateMessage),
  UpdatePrivateMessage(UpdatePrivateMessage),
  DeletePrivateMessage(DeletePrivateMessage),
  UndoDeletePrivateMessage(UndoDeletePrivateMessage),
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
  AnnounceActivity(AnnounceActivity),
  UpdateCommunity(UpdateCommunity),
  DeleteCommunity(DeleteCommunity),
  RemoveCommunity(RemoveCommunity),
  UndoDeleteCommunity(UndoDeleteCommunity),
  UndoRemoveCommunity(UndoRemoveCommunity),
  BlockUserFromCommunity(BlockUserFromCommunity),
  UndoBlockUserFromCommunity(UndoBlockUserFromCommunity),
}

// todo: can probably get rid of these?
#[async_trait::async_trait(?Send)]
impl VerifyActivity for SharedInboxActivities {
  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    self.verify(context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for SharedInboxActivities {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    self.receive(context, request_counter).await
  }
}
