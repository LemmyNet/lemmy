use crate::activities_new::{
  comment::{
    create::CreateComment,
    delete::DeleteComment,
    dislike::DislikeComment,
    like::LikeComment,
    remove::RemoveComment,
    update::UpdateComment,
  },
  community::{
    delete::DeleteCommunity,
    remove::RemoveCommunity,
    undo_delete::UndoDeleteCommunity,
    undo_remove::UndoRemoveCommunity,
    update::UpdateCommunity,
  },
  follow::AcceptFollowCommunity,
  post::{
    create::CreatePost,
    delete::DeletePost,
    dislike::DislikePost,
    like::LikePost,
    remove::RemovePost,
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
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity<Kind> {
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  id: Url,

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

// TODO: this is probably wrong, it contains all activities
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum PersonAcceptedActivitiesNew {
  AcceptFollowCommunity(AcceptFollowCommunity),
  CreatePrivateMessage(CreatePrivateMessage),
  UpdatePrivateMessage(UpdatePrivateMessage),
  DeletePrivateMessage(DeletePrivateMessage),
  UndoDeletePrivateMessage(UndoDeletePrivateMessage),
  CreateComment(CreateComment),
  UpdateComment(UpdateComment),
  LikeComment(LikeComment),
  DislikeComment(DislikeComment),
  DeleteComment(DeleteComment),
  RemoveComment(RemoveComment),
  CreatePost(CreatePost),
  UpdatePost(UpdatePost),
  LikePost(LikePost),
  DislikePost(DislikePost),
  DeletePost(DeletePost),
  RemovePost(RemovePost),
  UpdateCommunity(UpdateCommunity),
  DeleteCommunity(DeleteCommunity),
  RemoveCommunity(RemoveCommunity),
  UndoDeleteCommunity(UndoDeleteCommunity),
  UndoRemoveCommunity(UndoRemoveCommunity),
}

// todo: can probably get rid of these?
#[async_trait::async_trait(?Send)]
impl VerifyActivity for PersonAcceptedActivitiesNew {
  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    self.verify(context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for PersonAcceptedActivitiesNew {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    self.receive(context, request_counter).await
  }
}
