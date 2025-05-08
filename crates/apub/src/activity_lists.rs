use crate::protocol::activities::{
  block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
  community::{
    announce::{AnnounceActivity, RawAnnouncableActivities},
    collection_add::CollectionAdd,
    collection_remove::CollectionRemove,
    lock_page::{LockPage, UndoLockPage},
    report::Report,
    resolve_report::ResolveReport,
    update::UpdateCommunity,
  },
  create_or_update::{note_wrapper::CreateOrUpdateNoteWrapper, page::CreateOrUpdatePage},
  deletion::{delete::Delete, undo_delete::UndoDelete},
  following::{
    accept::AcceptFollow,
    follow::Follow,
    reject::RejectFollow,
    undo_follow::UndoFollow,
  },
  voting::{undo_vote::UndoVote, vote::Vote},
};
use activitypub_federation::{config::Data, traits::ActivityHandler};
use lemmy_api_common::context::LemmyContext;
use lemmy_apub_objects::{
  objects::community::ApubCommunity,
  protocol::page::Page,
  utils::protocol::InCommunity,
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde::{Deserialize, Serialize};
use url::Url;

/// List of activities which the shared inbox can handle.
///
/// This could theoretically be defined as an enum with variants `GroupInboxActivities` and
/// `PersonInboxActivities`. In practice we need to write it out manually so that priorities
/// are handled correctly.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum SharedInboxActivities {
  Follow(Follow),
  AcceptFollow(AcceptFollow),
  RejectFollow(RejectFollow),
  UndoFollow(UndoFollow),
  Report(Report),
  ResolveReport(ResolveReport),
  AnnounceActivity(AnnounceActivity),
  /// This is a catch-all and needs to be last
  RawAnnouncableActivities(RawAnnouncableActivities),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum AnnouncableActivities {
  CreateOrUpdateNoteWrapper(CreateOrUpdateNoteWrapper),
  CreateOrUpdatePost(CreateOrUpdatePage),
  Vote(Vote),
  UndoVote(UndoVote),
  Delete(Delete),
  UndoDelete(UndoDelete),
  UpdateCommunity(UpdateCommunity),
  BlockUser(BlockUser),
  UndoBlockUser(UndoBlockUser),
  CollectionAdd(CollectionAdd),
  CollectionRemove(CollectionRemove),
  LockPost(LockPage),
  UndoLockPost(UndoLockPage),
  Report(Report),
  ResolveReport(ResolveReport),
  // For compatibility with Pleroma/Mastodon (send only)
  Page(Page),
}

impl InCommunity for AnnouncableActivities {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    use AnnouncableActivities::*;
    match self {
      CreateOrUpdateNoteWrapper(a) => a.community(context).await,
      CreateOrUpdatePost(a) => a.community(context).await,
      Vote(a) => a.community(context).await,
      UndoVote(a) => a.community(context).await,
      Delete(a) => a.community(context).await,
      UndoDelete(a) => a.community(context).await,
      UpdateCommunity(a) => a.community(context).await,
      BlockUser(a) => a.community(context).await,
      UndoBlockUser(a) => a.community(context).await,
      CollectionAdd(a) => a.community(context).await,
      CollectionRemove(a) => a.community(context).await,
      LockPost(a) => a.community(context).await,
      UndoLockPost(a) => a.community(context).await,
      Report(a) => a.community(context).await,
      ResolveReport(a) => a.community(context).await,
      Page(_) => Err(LemmyErrorType::NotFound.into()),
    }
  }
}

#[cfg(test)]
mod tests {

  use crate::activity_lists::SharedInboxActivities;
  use lemmy_apub_objects::utils::test::{test_json, test_parse_lemmy_item};
  use lemmy_utils::error::LemmyResult;

  #[test]
  fn test_shared_inbox() -> LemmyResult<()> {
    test_parse_lemmy_item::<SharedInboxActivities>(
      "assets/lemmy/activities/deletion/delete_user.json",
    )?;
    test_parse_lemmy_item::<SharedInboxActivities>(
      "assets/lemmy/activities/following/accept.json",
    )?;
    test_parse_lemmy_item::<SharedInboxActivities>(
      "assets/lemmy/activities/create_or_update/create_comment.json",
    )?;
    test_parse_lemmy_item::<SharedInboxActivities>(
      "assets/lemmy/activities/create_or_update/create_private_message.json",
    )?;
    test_parse_lemmy_item::<SharedInboxActivities>(
      "assets/lemmy/activities/following/follow.json",
    )?;
    test_parse_lemmy_item::<SharedInboxActivities>(
      "assets/lemmy/activities/create_or_update/create_comment.json",
    )?;
    test_json::<SharedInboxActivities>("assets/mastodon/activities/follow.json")?;
    Ok(())
  }
}
