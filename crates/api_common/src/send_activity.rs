use crate::context::LemmyContext;
use activitypub_federation::config::Data;
use futures::future::BoxFuture;
use lemmy_db_schema::source::post::Post;
use lemmy_utils::{error::LemmyResult, SYNCHRONOUS_FEDERATION};
use once_cell::sync::{Lazy, OnceCell};
use tokio::sync::{
  mpsc,
  mpsc::{UnboundedReceiver, UnboundedSender},
  Mutex,
};

type MatchOutgoingActivitiesBoxed =
  Box<for<'a> fn(SendActivityData, &'a Data<LemmyContext>) -> BoxFuture<'a, LemmyResult<()>>>;

/// This static is necessary so that activities can be sent out synchronously for tests.
pub static MATCH_OUTGOING_ACTIVITIES: OnceCell<MatchOutgoingActivitiesBoxed> = OnceCell::new();

#[derive(Debug)]
pub enum SendActivityData {
  CreatePost(Post),
}

static ACTIVITY_CHANNEL: Lazy<ActivityChannel> = Lazy::new(|| {
  let (sender, receiver) = mpsc::unbounded_channel();
  ActivityChannel {
    sender,
    receiver: Mutex::new(receiver),
  }
});

pub struct ActivityChannel {
  sender: UnboundedSender<SendActivityData>,
  receiver: Mutex<UnboundedReceiver<SendActivityData>>,
}

impl ActivityChannel {
  pub async fn retrieve_activity() -> Option<SendActivityData> {
    let mut lock = ACTIVITY_CHANNEL.receiver.lock().await;
    lock.recv().await
  }

  pub async fn submit_activity(
    data: SendActivityData,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    if *SYNCHRONOUS_FEDERATION {
      MATCH_OUTGOING_ACTIVITIES
        .get()
        .expect("retrieve function pointer")(data, context)
      .await?;
    } else {
      let lock = &ACTIVITY_CHANNEL.sender;
      lock.send(data)?;
    }
    Ok(())
  }
}
