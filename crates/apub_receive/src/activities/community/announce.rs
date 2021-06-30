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
    community::{block_user::BlockUserFromCommunity, undo_block_user::UndoBlockUserFromCommunity},
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
    LemmyActivity,
  },
  http::is_activity_already_known,
};
use activitystreams::activity::kind::RemoveType;
use lemmy_apub::{check_is_apub_id_valid, fetcher::person::get_or_fetch_and_upsert_person};
use lemmy_apub_lib::{verify_domains_match, ActivityHandler, PublicUrl};
use lemmy_db_schema::source::{community::Community, person::Person};
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
  BlockUserFromCommunity(BlockUserFromCommunity),
  UndoBlockUserFromCommunity(UndoBlockUserFromCommunity),
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for AnnouncableActivities {
  type Actor = Person;

  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    self.verify(context).await
  }

  async fn receive(
    &self,
    actor: Self::Actor,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    self.receive(actor, context, request_counter).await
  }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnounceActivity {
  to: PublicUrl,
  object: LemmyActivity<AnnouncableActivities>,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: RemoveType,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for LemmyActivity<AnnounceActivity> {
  type Actor = Community;

  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.actor, self.id_unchecked())?;
    verify_domains_match(&self.actor, &self.inner.cc[0])?;
    check_is_apub_id_valid(&self.actor, false)?;
    self.inner.object.inner.verify(context).await
  }

  async fn receive(
    &self,
    _actor: Self::Actor,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    if is_activity_already_known(context.pool(), &self.inner.object.id_unchecked()).await? {
      return Ok(());
    }
    let inner_actor =
      get_or_fetch_and_upsert_person(&self.inner.object.actor, context, request_counter).await?;
    self
      .inner
      .object
      .inner
      .receive(inner_actor, context, request_counter)
      .await
  }
}
