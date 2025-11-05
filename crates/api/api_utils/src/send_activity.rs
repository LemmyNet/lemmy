use crate::context::LemmyContext;
use activitypub_federation::config::Data;
use either::Either;
use lemmy_db_schema::{
  newtypes::{CommunityId, DbUrl, PersonId},
  source::{
    comment::Comment,
    community::Community,
    multi_community::MultiCommunity,
    person::Person,
    post::Post,
    private_message::PrivateMessage,
    site::Site,
  },
};
use lemmy_db_views_community::api::BanFromCommunity;
use lemmy_db_views_post::api::DeletePost;
use lemmy_db_views_private_message::PrivateMessageView;
use lemmy_utils::error::LemmyResult;
use std::sync::LazyLock;
use tokio::{
  sync::{
    Mutex,
    mpsc,
    mpsc::{UnboundedReceiver, UnboundedSender, WeakUnboundedSender},
  },
  task::JoinHandle,
};
use url::Url;

#[derive(Debug)]
pub enum SendActivityData {
  CreatePost(Post),
  UpdatePost(Post),
  DeletePost(Post, Person, DeletePost),
  RemovePost {
    post: Post,
    moderator: Person,
    reason: String,
    removed: bool,
  },
  LockPost(Post, Person, bool, String),
  FeaturePost(Post, Person, bool),
  CreateComment(Comment),
  UpdateComment(Comment),
  DeleteComment(Comment, Person, Community),
  RemoveComment {
    comment: Comment,
    moderator: Person,
    community: Community,
    reason: String,
  },
  LockComment(Comment, Person, bool, String),
  LikePostOrComment {
    object_id: DbUrl,
    actor: Person,
    community: Community,
    previous_is_upvote: Option<bool>,
    new_is_upvote: Option<bool>,
  },
  FollowCommunity(Community, Person, bool),
  FollowMultiCommunity(MultiCommunity, Person, bool),
  AcceptFollower(CommunityId, PersonId),
  RejectFollower(CommunityId, PersonId),
  UpdateCommunity(Person, Community),
  DeleteCommunity(Person, Community, bool),
  RemoveCommunity {
    moderator: Person,
    community: Community,
    reason: String,
    removed: bool,
  },
  AddModToCommunity {
    moderator: Person,
    community_id: CommunityId,
    target: PersonId,
    added: bool,
  },
  BanFromCommunity {
    moderator: Person,
    community_id: CommunityId,
    target: Person,
    data: BanFromCommunity,
  },
  BanFromSite {
    moderator: Person,
    banned_user: Person,
    reason: String,
    remove_or_restore_data: Option<bool>,
    ban: bool,
    expires_at: Option<i64>,
  },
  CreatePrivateMessage(PrivateMessageView),
  UpdatePrivateMessage(PrivateMessageView),
  DeletePrivateMessage(Person, PrivateMessage, bool),
  DeleteUser(Person, bool),
  CreateReport {
    object_id: Url,
    actor: Person,
    receiver: Either<Site, Community>,
    reason: String,
  },
  SendResolveReport {
    object_id: Url,
    actor: Person,
    report_creator: Person,
    receiver: Either<Site, Community>,
  },
  UpdateMultiCommunity(MultiCommunity, Person),
}

// TODO: instead of static, move this into LemmyContext. make sure that stopping the process with
//       ctrl+c still works.
static ACTIVITY_CHANNEL: LazyLock<ActivityChannel> = LazyLock::new(|| {
  let (sender, receiver) = mpsc::unbounded_channel();
  let weak_sender = sender.downgrade();
  ActivityChannel {
    weak_sender,
    receiver: Mutex::new(receiver),
    keepalive_sender: Mutex::new(Some(sender)),
  }
});

pub struct ActivityChannel {
  weak_sender: WeakUnboundedSender<SendActivityData>,
  receiver: Mutex<UnboundedReceiver<SendActivityData>>,
  keepalive_sender: Mutex<Option<UnboundedSender<SendActivityData>>>,
}

impl ActivityChannel {
  pub async fn retrieve_activity() -> Option<SendActivityData> {
    let mut lock = ACTIVITY_CHANNEL.receiver.lock().await;
    lock.recv().await
  }

  pub fn submit_activity(data: SendActivityData, _context: &Data<LemmyContext>) -> LemmyResult<()> {
    // could do `ACTIVITY_CHANNEL.keepalive_sender.lock()` instead and get rid of weak_sender,
    // not sure which way is more efficient
    if let Some(sender) = ACTIVITY_CHANNEL.weak_sender.upgrade() {
      sender.send(data)?;
    }
    Ok(())
  }

  pub async fn close(outgoing_activities_task: JoinHandle<()>) -> LemmyResult<()> {
    ACTIVITY_CHANNEL.keepalive_sender.lock().await.take();
    outgoing_activities_task.await?;
    Ok(())
  }
}
