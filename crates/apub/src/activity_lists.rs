use crate::{
  objects::community::ApubCommunity,
  protocol::{
    activities::{
      block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
      community::{
        announce::{AnnounceActivity, RawAnnouncableActivities},
        collection_add::CollectionAdd,
        collection_remove::CollectionRemove,
        lock_page::{LockPage, UndoLockPage},
        report::Report,
        update::UpdateCommunity,
      },
      create_or_update::{
        chat_message::CreateOrUpdateChatMessage,
        note::CreateOrUpdateNote,
        page::CreateOrUpdatePage,
      },
      deletion::{delete::Delete, undo_delete::UndoDelete},
      following::{accept::AcceptFollow, follow::Follow, undo_follow::UndoFollow},
      voting::{undo_vote::UndoVote, vote::Vote},
    },
    objects::page::Page,
    InCommunity,
  },
};
use activitypub_federation::{config::Data, traits::ActivityHandler};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyError;
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
  UndoFollow(UndoFollow),
  CreateOrUpdatePrivateMessage(CreateOrUpdateChatMessage),
  Report(Report),
  AnnounceActivity(AnnounceActivity),
  /// This is a catch-all and needs to be last
  RawAnnouncableActivities(RawAnnouncableActivities),
}

/// List of activities which the group inbox can handle.
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum GroupInboxActivities {
  Follow(Follow),
  UndoFollow(UndoFollow),
  Report(Report),
  /// This is a catch-all and needs to be last
  AnnouncableActivities(RawAnnouncableActivities),
}

/// List of activities which the person inbox can handle.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum PersonInboxActivities {
  Follow(Follow),
  AcceptFollow(AcceptFollow),
  UndoFollow(UndoFollow),
  CreateOrUpdatePrivateMessage(CreateOrUpdateChatMessage),
  Delete(Delete),
  UndoDelete(UndoDelete),
  AnnounceActivity(AnnounceActivity),
  /// User can also receive some "announcable" activities, eg a comment mention.
  AnnouncableActivities(AnnouncableActivities),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum AnnouncableActivities {
  CreateOrUpdateComment(CreateOrUpdateNote),
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
  // For compatibility with Pleroma/Mastodon (send only)
  Page(Page),
}

#[async_trait::async_trait]
impl InCommunity for AnnouncableActivities {
  #[tracing::instrument(skip(self, context))]
  async fn community(&self, context: &Data<LemmyContext>) -> Result<ApubCommunity, LemmyError> {
    use AnnouncableActivities::*;
    match self {
      CreateOrUpdateComment(a) => a.community(context).await,
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
      Page(_) => unimplemented!(),
    }
  }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use crate::{
    activity_lists::{GroupInboxActivities, PersonInboxActivities, SharedInboxActivities},
    protocol::tests::{test_json, test_parse_lemmy_item},
  };
  use lemmy_utils::error::LemmyResult;

  #[test]
  fn test_group_inbox() -> LemmyResult<()> {
    test_parse_lemmy_item::<GroupInboxActivities>("assets/lemmy/activities/following/follow.json")?;
    test_parse_lemmy_item::<GroupInboxActivities>(
      "assets/lemmy/activities/create_or_update/create_note.json",
    )?;
    Ok(())
  }

  #[test]
  fn test_person_inbox() -> LemmyResult<()> {
    test_parse_lemmy_item::<PersonInboxActivities>(
      "assets/lemmy/activities/following/accept.json",
    )?;
    test_parse_lemmy_item::<PersonInboxActivities>(
      "assets/lemmy/activities/create_or_update/create_note.json",
    )?;
    test_parse_lemmy_item::<PersonInboxActivities>(
      "assets/lemmy/activities/create_or_update/create_private_message.json",
    )?;
    test_json::<PersonInboxActivities>("assets/mastodon/activities/follow.json")?;
    Ok(())
  }

  #[test]
  fn test_shared_inbox() -> LemmyResult<()> {
    test_parse_lemmy_item::<SharedInboxActivities>(
      "assets/lemmy/activities/deletion/delete_user.json",
    )?;
    Ok(())
  }
}
