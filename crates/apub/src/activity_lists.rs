use crate::{
  activities::{community::announce::GetCommunity, verify_person_in_community},
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
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  deser::context::WithContext,
  traits::{activity_handler, ActivityHandler},
};
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[activity_handler(LemmyContext, LemmyError)]
pub enum SharedInboxActivities {
  GroupInboxActivities(Box<WithContext<GroupInboxActivities>>),
  // Note, pm activities need to be at the end, otherwise comments will end up here. We can probably
  // avoid this problem by replacing createpm.object with our own struct, instead of NoteExt.
  PersonInboxActivities(Box<WithContext<PersonInboxActivities>>),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum GroupInboxActivities {
  FollowCommunity(FollowCommunity),
  UndoFollowCommunity(UndoFollowCommunity),
  AnnouncableActivities(Box<AnnouncableActivities>),
  Report(Report),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[activity_handler(LemmyContext, LemmyError)]
pub enum PersonInboxActivities {
  AcceptFollowCommunity(AcceptFollowCommunity),
  /// Some activities can also be sent from user to user, eg a comment with mentions
  AnnouncableActivities(AnnouncableActivities),
  CreateOrUpdatePrivateMessage(CreateOrUpdatePrivateMessage),
  Delete(Delete),
  UndoDelete(UndoDelete),
  AnnounceActivity(AnnounceActivity),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[activity_handler(LemmyContext, LemmyError)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[activity_handler(LemmyContext, LemmyError)]
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
  fn object_id(&self) -> &Url {
    ActivityHandler::id(self)
  }
}

// Need to implement this manually to announce matching activities
#[async_trait::async_trait(?Send)]
impl ActivityHandler for GroupInboxActivities {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    match self {
      GroupInboxActivities::FollowCommunity(a) => a.id(),
      GroupInboxActivities::UndoFollowCommunity(a) => a.id(),
      GroupInboxActivities::AnnouncableActivities(a) => a.object_id(),
      GroupInboxActivities::Report(a) => a.id(),
    }
  }

  fn actor(&self) -> &Url {
    match self {
      GroupInboxActivities::FollowCommunity(a) => a.actor(),
      GroupInboxActivities::UndoFollowCommunity(a) => a.actor(),
      GroupInboxActivities::AnnouncableActivities(a) => a.actor(),
      GroupInboxActivities::Report(a) => a.actor(),
    }
  }

  async fn verify(
    &self,
    data: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    match self {
      GroupInboxActivities::FollowCommunity(a) => a.verify(data, request_counter).await,
      GroupInboxActivities::UndoFollowCommunity(a) => a.verify(data, request_counter).await,
      GroupInboxActivities::AnnouncableActivities(a) => a.verify(data, request_counter).await,
      GroupInboxActivities::Report(a) => a.verify(data, request_counter).await,
    }
  }

  async fn receive(
    self,
    data: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    match self {
      GroupInboxActivities::FollowCommunity(a) => a.receive(data, request_counter).await,
      GroupInboxActivities::UndoFollowCommunity(a) => a.receive(data, request_counter).await,
      GroupInboxActivities::AnnouncableActivities(activity) => {
        activity.clone().receive(data, request_counter).await?;

        // Ignore failures in get_community(). those happen because Delete/PrivateMessage is not in a
        // community, but looks identical to Delete/Post or Delete/Comment which are in a community.
        let community = activity.get_community(data, &mut 0).await;
        if let Ok(community) = community {
          if community.local {
            let actor_id = ObjectId::new(activity.actor().clone());
            verify_person_in_community(&actor_id, &community, data, &mut 0).await?;
            AnnounceActivity::send(*activity, &community, data).await?;
          }
        }
        Ok(())
      }
      GroupInboxActivities::Report(a) => a.receive(data, request_counter).await,
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
