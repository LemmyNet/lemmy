use crate::activities::{
  comment::{
    create::CreateComment,
    delete::DeleteComment,
    remove::RemoveComment,
    undo_delete::UndoDeleteComment,
    undo_remove::UndoRemoveComment,
    update::UpdateComment,
  },
  community::{
    add_mod::AddMod,
    announce::AnnounceActivity,
    block_user::BlockUserFromCommunity,
    delete::DeleteCommunity,
    remove::RemoveCommunity,
    remove_mod::RemoveMod,
    undo_block_user::UndoBlockUserFromCommunity,
    undo_delete::UndoDeleteCommunity,
    undo_remove::UndoRemoveCommunity,
    update::UpdateCommunity,
  },
  following::{accept::AcceptFollowCommunity, follow::FollowCommunity, undo::UndoFollowCommunity},
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
  private_message::{
    create::CreatePrivateMessage,
    delete::DeletePrivateMessage,
    undo_delete::UndoDeletePrivateMessage,
    update::UpdatePrivateMessage,
  },
};
use lemmy_apub_lib::{ActivityCommonFields, ActivityHandlerNew};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandlerNew)]
#[serde(untagged)]
pub enum PersonInboxActivities {
  AcceptFollowCommunity(AcceptFollowCommunity),
  CreatePrivateMessage(CreatePrivateMessage),
  UpdatePrivateMessage(UpdatePrivateMessage),
  DeletePrivateMessage(DeletePrivateMessage),
  UndoDeletePrivateMessage(UndoDeletePrivateMessage),
  AnnounceActivity(Box<AnnounceActivity>),
}

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandlerNew)]
#[serde(untagged)]
pub enum GroupInboxActivities {
  FollowCommunity(FollowCommunity),
  UndoFollowCommunity(UndoFollowCommunity),
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
  UpdateCommunity(Box<UpdateCommunity>),
  DeleteCommunity(DeleteCommunity),
  RemoveCommunity(RemoveCommunity),
  UndoDeleteCommunity(UndoDeleteCommunity),
  UndoRemoveCommunity(UndoRemoveCommunity),
  BlockUserFromCommunity(BlockUserFromCommunity),
  UndoBlockUserFromCommunity(UndoBlockUserFromCommunity),
  AddMod(AddMod),
  RemoveMod(RemoveMod),
}

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandlerNew)]
#[serde(untagged)]
pub enum SharedInboxActivities {
  // received by group
  FollowCommunity(FollowCommunity),
  UndoFollowCommunity(UndoFollowCommunity),
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
  UndoDislikePostOrComment(UndoDislikePostOrComment),
  UndoLikePostOrComment(UndoLikePostOrComment),
  UpdateCommunity(Box<UpdateCommunity>),
  DeleteCommunity(DeleteCommunity),
  RemoveCommunity(RemoveCommunity),
  UndoDeleteCommunity(UndoDeleteCommunity),
  UndoRemoveCommunity(UndoRemoveCommunity),
  BlockUserFromCommunity(BlockUserFromCommunity),
  UndoBlockUserFromCommunity(UndoBlockUserFromCommunity),
  AddMod(AddMod),
  RemoveMod(RemoveMod),
  // received by person
  AcceptFollowCommunity(AcceptFollowCommunity),
  // Note, pm activities need to be at the end, otherwise comments will end up here. We can probably
  // avoid this problem by replacing createpm.object with our own struct, instead of NoteExt.
  CreatePrivateMessage(CreatePrivateMessage),
  UpdatePrivateMessage(UpdatePrivateMessage),
  DeletePrivateMessage(DeletePrivateMessage),
  UndoDeletePrivateMessage(UndoDeletePrivateMessage),
  AnnounceActivity(Box<AnnounceActivity>),
}
