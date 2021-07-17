use crate::activities::{
  comment::{create::CreateComment, update::UpdateComment},
  community::{
    add_mod::AddMod,
    announce::AnnounceActivity,
    block_user::BlockUserFromCommunity,
    undo_block_user::UndoBlockUserFromCommunity,
    update::UpdateCommunity,
  },
  deletion::{delete::DeletePostCommentOrCommunity, undo_delete::UndoDeletePostCommentOrCommunity},
  following::{accept::AcceptFollowCommunity, follow::FollowCommunity, undo::UndoFollowCommunity},
  post::{create::CreatePost, update::UpdatePost},
  private_message::{
    create::CreatePrivateMessage,
    delete::DeletePrivateMessage,
    undo_delete::UndoDeletePrivateMessage,
    update::UpdatePrivateMessage,
  },
  removal::{
    remove::RemovePostCommentCommunityOrMod,
    undo_remove::UndoRemovePostCommentOrCommunity,
  },
  voting::{
    dislike::DislikePostOrComment,
    like::LikePostOrComment,
    undo_dislike::UndoDislikePostOrComment,
    undo_like::UndoLikePostOrComment,
  },
};
use lemmy_apub_lib::{ActivityCommonFields, ActivityHandler};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler)]
#[serde(untagged)]
pub enum PersonInboxActivities {
  AcceptFollowCommunity(AcceptFollowCommunity),
  CreatePrivateMessage(CreatePrivateMessage),
  UpdatePrivateMessage(UpdatePrivateMessage),
  DeletePrivateMessage(DeletePrivateMessage),
  UndoDeletePrivateMessage(UndoDeletePrivateMessage),
  AnnounceActivity(Box<AnnounceActivity>),
}

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler)]
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
  DeletePostCommentOrCommunity(DeletePostCommentOrCommunity),
  UndoDeletePostCommentOrCommunity(UndoDeletePostCommentOrCommunity),
  RemovePostCommentOrCommunity(RemovePostCommentCommunityOrMod),
  UndoRemovePostCommentOrCommunity(UndoRemovePostCommentOrCommunity),
  UpdateCommunity(Box<UpdateCommunity>),
  BlockUserFromCommunity(BlockUserFromCommunity),
  UndoBlockUserFromCommunity(UndoBlockUserFromCommunity),
  AddMod(AddMod),
}

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler)]
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
  DeletePostCommentOrCommunity(DeletePostCommentOrCommunity),
  UndoDeletePostCommentOrCommunity(UndoDeletePostCommentOrCommunity),
  RemovePostCommentOrCommunity(RemovePostCommentCommunityOrMod),
  UndoRemovePostCommentOrCommunity(UndoRemovePostCommentOrCommunity),
  UpdateCommunity(Box<UpdateCommunity>),
  BlockUserFromCommunity(BlockUserFromCommunity),
  UndoBlockUserFromCommunity(UndoBlockUserFromCommunity),
  AddMod(AddMod),
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
