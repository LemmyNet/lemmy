use crate::activities::{
  comment::{create::CreateComment, update::UpdateComment},
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
  CreatePost(CreatePost),
  UpdatePost(UpdatePost),
  LikePostOrComment(LikePostOrComment),
  DislikePostOrComment(DislikePostOrComment),
  UndoLikePostOrComment(UndoLikePostOrComment),
  UndoDislikePostOrComment(UndoDislikePostOrComment),
  DeletePostOrComment(DeletePostOrComment),
  UndoDeletePostOrComment(UndoDeletePostOrComment),
  RemovePostOrComment(RemovePostOrComment),
  UndoRemovePostOrComment(UndoRemovePostOrComment),
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
  CreatePost(CreatePost),
  UpdatePost(UpdatePost),
  LikePostOrComment(LikePostOrComment),
  DislikePostOrComment(DislikePostOrComment),
  UndoDislikePostOrComment(UndoDislikePostOrComment),
  UndoLikePostOrComment(UndoLikePostOrComment),
  DeletePostOrComment(DeletePostOrComment),
  UndoDeletePostOrComment(UndoDeletePostOrComment),
  RemovePostOrComment(RemovePostOrComment),
  UndoRemovePostOrComment(UndoRemovePostOrComment),
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
