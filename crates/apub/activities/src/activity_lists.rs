use crate::protocol::{
  block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
  community::{
    announce::{AnnounceActivity, RawAnnouncableActivities},
    collection_add::CollectionAdd,
    collection_remove::CollectionRemove,
    lock::{LockPageOrNote, UndoLockPageOrNote},
    report::Report,
    resolve_report::ResolveReport,
    update::Update,
    warn::Warn,
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
use activitypub_federation::{config::Data, traits::Activity};
use lemmy_api_utils::context::LemmyContext;
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
#[enum_delegate::implement(Activity)]
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
#[enum_delegate::implement(Activity)]
pub enum AnnouncableActivities {
  CreateOrUpdateNoteWrapper(CreateOrUpdateNoteWrapper),
  CreateOrUpdatePost(CreateOrUpdatePage),
  Vote(Vote),
  UndoVote(UndoVote),
  Delete(Delete),
  UndoDelete(UndoDelete),
  UpdateCommunity(Box<Update>),
  BlockUser(BlockUser),
  UndoBlockUser(UndoBlockUser),
  CollectionAdd(CollectionAdd),
  CollectionRemove(CollectionRemove),
  Lock(LockPageOrNote),
  UndoLock(UndoLockPageOrNote),
  Report(Report),
  ResolveReport(ResolveReport),
  Warn(Warn),
  InnerActivities(InnerActivities),
  // For compatibility with Pleroma/Mastodon (send only)
  Page(Page),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InnerActivities {
  id: Url,
  total_activities: i32,
  activities: Vec<AnnouncableActivities>,
}

impl Activity for InnerActivities {
  #[doc = " App data type passed to handlers. Must be identical to"]
  #[doc = " [crate::config::FederationConfigBuilder::app_data] type."]
  type DataType;

  #[doc = " Error type returned by handler methods"]
  type Error;

  #[doc = " `id` field of the activity"]
  fn id(&self) -> &Url {
    todo!()
  }

  #[doc = " `actor` field of activity"]
  fn actor(&self) -> &Url {
    todo!()
  }

  #[doc = " Verifies that the received activity is valid."]
  #[doc = ""]
  #[doc = " This needs to be a separate method, because it might be used for activities"]
  #[doc = " like `Undo/Follow`, which shouldn\'t perform any database write for the inner `Follow`."]
  #[must_use]
  #[allow(
    elided_named_lifetimes,
    clippy::type_complexity,
    clippy::type_repetition_in_bounds
  )]
  fn verify<'life0, 'life1, 'async_trait>(
    &'life0 self,
    data: &'life1 Data<Self::DataType>,
  ) -> ::core::pin::Pin<
    Box<
      dyn ::core::future::Future<Output = Result<(), Self::Error>>
        + ::core::marker::Send
        + 'async_trait,
    >,
  >
  where
    'life0: 'async_trait,
    'life1: 'async_trait,
    Self: 'async_trait,
  {
    todo!()
  }

  #[doc = " Called when an activity is received."]
  #[doc = ""]
  #[doc = " Should perform validation and possibly write action to the database. In case the activity"]
  #[doc = " has a nested `object` field, must call `object.from_json` handler."]
  #[must_use]
  #[allow(
    elided_named_lifetimes,
    clippy::type_complexity,
    clippy::type_repetition_in_bounds
  )]
  fn receive<'life0, 'async_trait>(
    self,
    data: &'life0 Data<Self::DataType>,
  ) -> ::core::pin::Pin<
    Box<
      dyn ::core::future::Future<Output = Result<(), Self::Error>>
        + ::core::marker::Send
        + 'async_trait,
    >,
  >
  where
    'life0: 'async_trait,
    Self: 'async_trait,
  {
    todo!()
  }
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
      Lock(a) => a.community(context).await,
      UndoLock(a) => a.community(context).await,
      Report(a) => a.community(context).await,
      ResolveReport(a) => a.community(context).await,
      Warn(a) => a.community(context).await,
      Page(_) => Err(LemmyErrorType::NotFound.into()),
      InnerActivities(_) => Err(LemmyErrorType::NotFound.into()),
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
      "../apub/assets/lemmy/activities/deletion/delete_user.json",
    )?;
    test_parse_lemmy_item::<SharedInboxActivities>(
      "../apub/assets/lemmy/activities/following/accept.json",
    )?;
    test_parse_lemmy_item::<SharedInboxActivities>(
      "../apub/assets/lemmy/activities/create_or_update/create_comment.json",
    )?;
    test_parse_lemmy_item::<SharedInboxActivities>(
      "../apub/assets/lemmy/activities/create_or_update/create_private_message.json",
    )?;
    test_parse_lemmy_item::<SharedInboxActivities>(
      "../apub/assets/lemmy/activities/following/follow.json",
    )?;
    test_parse_lemmy_item::<SharedInboxActivities>(
      "../apub/assets/lemmy/activities/create_or_update/create_comment.json",
    )?;
    test_json::<SharedInboxActivities>("../apub/assets/mastodon/activities/follow.json")?;
    Ok(())
  }
}
