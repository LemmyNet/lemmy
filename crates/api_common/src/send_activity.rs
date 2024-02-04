use crate::{community::BanFromCommunity, context::LemmyContext, post::DeletePost};
use activitypub_federation::config::Data;
use futures::future::BoxFuture;
use lemmy_db_schema::{
  newtypes::{CommunityId, DbUrl, PersonId},
  source::{
    comment::Comment,
    community::Community,
    person::Person,
    post::Post,
    private_message::PrivateMessage,
  },
};
use lemmy_db_views::structs::PrivateMessageView;
use lemmy_utils::error::LemmyResult;
use once_cell::sync::{Lazy, OnceCell};
use tokio::{
  sync::{
    mpsc,
    mpsc::{UnboundedReceiver, UnboundedSender, WeakUnboundedSender},
    Mutex,
  },
  task::JoinHandle,
};
use url::Url;

type MatchOutgoingActivitiesBoxed =
  Box<for<'a> fn(SendActivityData, &'a Data<LemmyContext>) -> BoxFuture<'a, LemmyResult<()>>>;

/// This static is necessary so that the api_common crates don't need to depend on lemmy_apub
pub static MATCH_OUTGOING_ACTIVITIES: OnceCell<MatchOutgoingActivitiesBoxed> = OnceCell::new();

#[derive(Debug)]
pub enum SendActivityData {
  CreatePost(Post),
  UpdatePost(Post),
  DeletePost(Post, Person, DeletePost),
  RemovePost {
    post: Post,
    moderator: Person,
    reason: Option<String>,
    removed: bool,
  },
  LockPost(Post, Person, bool),
  FeaturePost(Post, Person, bool),
  CreateComment(Comment),
  UpdateComment(Comment),
  DeleteComment(Comment, Person, Community),
  RemoveComment {
    comment: Comment,
    moderator: Person,
    community: Community,
    reason: Option<String>,
  },
  LikePostOrComment {
    object_id: DbUrl,
    actor: Person,
    community: Community,
    score: i16,
  },
  FollowCommunity(Community, Person, bool),
  UpdateCommunity(Person, Community),
  DeleteCommunity(Person, Community, bool),
  RemoveCommunity {
    moderator: Person,
    community: Community,
    reason: Option<String>,
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
    reason: Option<String>,
    remove_data: Option<bool>,
    ban: bool,
    expires: Option<i64>,
  },
  CreatePrivateMessage(PrivateMessageView),
  UpdatePrivateMessage(PrivateMessageView),
  DeletePrivateMessage(Person, PrivateMessage, bool),
  DeleteUser(Person, bool),
  CreateReport {
    object_id: Url,
    actor: Person,
    community: Community,
    reason: String,
  },
}

// TODO: instead of static, move this into LemmyContext. make sure that stopping the process with
//       ctrl+c still works.
static ACTIVITY_CHANNEL: Lazy<ActivityChannel> = Lazy::new(|| {
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

  pub async fn submit_activity(
    data: SendActivityData,
    _context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
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
