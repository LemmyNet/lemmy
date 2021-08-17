use crate::activities::{
  comment::create_or_update::CreateOrUpdateComment,
  community::{
    add_mod::AddMod,
    announce::AnnounceActivity,
    block_user::BlockUserFromCommunity,
    undo_block_user::UndoBlockUserFromCommunity,
    update::UpdateCommunity,
  },
  deletion::{delete::Delete, undo_delete::UndoDelete},
  following::{accept::AcceptFollowCommunity, follow::FollowCommunity, undo::UndoFollowCommunity},
  post::create_or_update::CreateOrUpdatePost,
  private_message::{
    create_or_update::CreateOrUpdatePrivateMessage,
    delete::DeletePrivateMessage,
    undo_delete::UndoDeletePrivateMessage,
  },
  removal::{remove::RemoveMod, undo_remove::UndoRemovePostCommentOrCommunity},
  voting::{undo_vote::UndoVote, vote::Vote},
};
use lemmy_apub_lib::{ActivityCommonFields, ActivityHandler};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler)]
#[serde(untagged)]
pub enum PersonInboxActivities {
  AcceptFollowCommunity(AcceptFollowCommunity),
  CreateOrUpdatePrivateMessage(CreateOrUpdatePrivateMessage),
  DeletePrivateMessage(DeletePrivateMessage),
  UndoDeletePrivateMessage(UndoDeletePrivateMessage),
  AnnounceActivity(Box<AnnounceActivity>),
}

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler)]
#[serde(untagged)]
pub enum GroupInboxActivities {
  FollowCommunity(FollowCommunity),
  UndoFollowCommunity(UndoFollowCommunity),
  CreateOrUpdateComment(CreateOrUpdateComment),
  CreateOrUpdatePost(Box<CreateOrUpdatePost>),
  Vote(Vote),
  UndoVote(UndoVote),
  DeletePostCommentOrCommunity(Delete),
  UndoDeletePostCommentOrCommunity(UndoDelete),
  UndoRemovePostCommentOrCommunity(UndoRemovePostCommentOrCommunity),
  UpdateCommunity(Box<UpdateCommunity>),
  BlockUserFromCommunity(BlockUserFromCommunity),
  UndoBlockUserFromCommunity(UndoBlockUserFromCommunity),
  AddMod(AddMod),
  RemoveMod(RemoveMod),
}

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler)]
#[serde(untagged)]
pub enum SharedInboxActivities {
  // received by group
  FollowCommunity(FollowCommunity),
  UndoFollowCommunity(UndoFollowCommunity),
  CreateOrUpdateComment(CreateOrUpdateComment),
  CreateOrUpdatePost(Box<CreateOrUpdatePost>),
  Vote(Vote),
  UndoVote(UndoVote),
  Delete(Delete),
  UndoDelete(UndoDelete),
  UndoRemovePostCommentOrCommunity(UndoRemovePostCommentOrCommunity),
  UpdateCommunity(Box<UpdateCommunity>),
  BlockUserFromCommunity(BlockUserFromCommunity),
  UndoBlockUserFromCommunity(UndoBlockUserFromCommunity),
  AddMod(AddMod),
  RemoveMod(RemoveMod),
  // received by person
  AcceptFollowCommunity(AcceptFollowCommunity),
  // Note, pm activities need to be at the end, otherwise comments will end up here. We can probably
  // avoid this problem by replacing createpm.object with our own struct, instead of NoteExt.
  CreateOrUpdatePrivateMessage(CreateOrUpdatePrivateMessage),
  DeletePrivateMessage(DeletePrivateMessage),
  UndoDeletePrivateMessage(UndoDeletePrivateMessage),
  AnnounceActivity(Box<AnnounceActivity>),
}
