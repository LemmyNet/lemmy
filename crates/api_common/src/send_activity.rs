use lemmy_db_schema::source::post::Post;
use lemmy_utils::error::LemmyResult;
use once_cell::sync::Lazy;
use tokio::sync::{
  mpsc,
  mpsc::{UnboundedReceiver, UnboundedSender},
  Mutex,
};

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

  pub async fn submit_activity(data: SendActivityData) -> LemmyResult<()> {
    // TODO: this will return immediately, and not wait for send to complete
    //       which causes problems for api tests
    let lock = &ACTIVITY_CHANNEL.sender;
    lock.send(data)?;
    Ok(())
  }

  pub async fn close() {
    ACTIVITY_CHANNEL.receiver.lock().await.close()
  }
}
