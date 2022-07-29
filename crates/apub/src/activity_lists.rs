use crate::{
  activities::community::announce::GetCommunity,
  objects::community::ApubCommunity,
  protocol::{
    activities::{
      block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
      community::{
        add_mod::AddMod,
        announce::AnnounceActivity,
        remove_mod::RemoveMod,
        report::Report,
        update::UpdateCommunity,
      },
      create_or_update::{
        comment::CreateOrUpdateComment,
        post::CreateOrUpdatePost,
        private_message::CreateOrUpdatePrivateMessage,
      },
      deletion::{delete::Delete, delete_user::DeleteUser, undo_delete::UndoDelete},
      following::{
        accept::AcceptFollowCommunity,
        follow::FollowCommunity,
        undo_follow::UndoFollowCommunity,
      },
      voting::{undo_vote::UndoVote, vote::Vote},
    },
    objects::page::Page,
    Id,
  },
};
use lemmy_apub_lib::traits::ActivityHandler;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler)]
#[serde(untagged)]
#[activity_handler(LemmyContext)]
pub enum SharedInboxActivities {
  GroupInboxActivities(Box<GroupInboxActivities>),
  // Note, pm activities need to be at the end, otherwise comments will end up here. We can probably
  // avoid this problem by replacing createpm.object with our own struct, instead of NoteExt.
  PersonInboxActivities(Box<PersonInboxActivities>),
}

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler)]
#[serde(untagged)]
#[activity_handler(LemmyContext)]
pub enum GroupInboxActivities {
  FollowCommunity(FollowCommunity),
  UndoFollowCommunity(UndoFollowCommunity),
  AnnouncableActivities(Box<AnnouncableActivities>),
  Report(Report),
}

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler)]
#[serde(untagged)]
#[activity_handler(LemmyContext)]
pub enum PersonInboxActivities {
  AcceptFollowCommunity(AcceptFollowCommunity),
  /// Some activities can also be sent from user to user, eg a comment with mentions
  AnnouncableActivities(AnnouncableActivities),
  CreateOrUpdatePrivateMessage(CreateOrUpdatePrivateMessage),
  Delete(Delete),
  UndoDelete(UndoDelete),
  AnnounceActivity(AnnounceActivity),
}

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler)]
#[serde(untagged)]
#[activity_handler(LemmyContext)]
pub enum AnnouncableActivities {
  CreateOrUpdateComment(CreateOrUpdateComment),
  CreateOrUpdatePost(Box<CreateOrUpdatePost>),
  Vote(Vote),
  UndoVote(UndoVote),
  Delete(Delete),
  UndoDelete(UndoDelete),
  UpdateCommunity(UpdateCommunity),
  BlockUser(BlockUser),
  UndoBlockUser(UndoBlockUser),
  AddMod(AddMod),
  RemoveMod(RemoveMod),
  // For compatibility with Pleroma/Mastodon (send only)
  Page(Page),
}

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler)]
#[serde(untagged)]
#[activity_handler(LemmyContext)]
#[allow(clippy::enum_variant_names)]
pub enum SiteInboxActivities {
  BlockUser(BlockUser),
  UndoBlockUser(UndoBlockUser),
  DeleteUser(DeleteUser),
}

#[async_trait::async_trait(?Send)]
impl GetCommunity for AnnouncableActivities {
  #[tracing::instrument(skip(self, context))]
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    use AnnouncableActivities::*;
    let community = match self {
      CreateOrUpdateComment(a) => a.get_community(context, request_counter).await?,
      CreateOrUpdatePost(a) => a.get_community(context, request_counter).await?,
      Vote(a) => a.get_community(context, request_counter).await?,
      UndoVote(a) => a.get_community(context, request_counter).await?,
      Delete(a) => a.get_community(context, request_counter).await?,
      UndoDelete(a) => a.get_community(context, request_counter).await?,
      UpdateCommunity(a) => a.get_community(context, request_counter).await?,
      BlockUser(a) => a.get_community(context, request_counter).await?,
      UndoBlockUser(a) => a.get_community(context, request_counter).await?,
      AddMod(a) => a.get_community(context, request_counter).await?,
      RemoveMod(a) => a.get_community(context, request_counter).await?,
      Page(_) => unimplemented!(),
    };
    Ok(community)
  }
}

impl Id for AnnouncableActivities {
  fn id(&self) -> &Url {
    use AnnouncableActivities::*;
    match self {
      CreateOrUpdateComment(c) => &c.id,
      CreateOrUpdatePost(c) => &c.id,
      Vote(v) => &v.id,
      UndoVote(u) => &u.id,
      Delete(d) => &d.id,
      UndoDelete(u) => &u.id,
      UpdateCommunity(u) => &u.id,
      BlockUser(b) => &b.id,
      UndoBlockUser(u) => &u.id,
      AddMod(a) => &a.id,
      RemoveMod(r) => &r.id,
      Page(p) => p.id.inner(),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    activity_lists::{GroupInboxActivities, PersonInboxActivities, SiteInboxActivities},
    protocol::tests::test_parse_lemmy_item,
  };

  #[test]
  fn test_group_inbox() {
    test_parse_lemmy_item::<GroupInboxActivities>("assets/lemmy/activities/following/follow.json")
      .unwrap();
    test_parse_lemmy_item::<GroupInboxActivities>(
      "assets/lemmy/activities/create_or_update/create_note.json",
    )
    .unwrap();
  }

  #[test]
  fn test_person_inbox() {
    test_parse_lemmy_item::<PersonInboxActivities>("assets/lemmy/activities/following/accept.json")
      .unwrap();
    test_parse_lemmy_item::<PersonInboxActivities>(
      "assets/lemmy/activities/create_or_update/create_note.json",
    )
    .unwrap();
    test_parse_lemmy_item::<PersonInboxActivities>(
      "assets/lemmy/activities/create_or_update/create_private_message.json",
    )
    .unwrap();
  }

  #[test]
  fn test_site_inbox() {
    test_parse_lemmy_item::<SiteInboxActivities>(
      "assets/lemmy/activities/deletion/delete_user.json",
    )
    .unwrap();
  }
}
