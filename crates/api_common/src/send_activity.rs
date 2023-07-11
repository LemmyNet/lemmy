use crate::context::LemmyContext;
use lemmy_db_schema::{newtypes::PersonId, source::post::Post};
use lemmy_utils::error::LemmyResult;
use once_cell::sync::Lazy;
use std::ops::Deref;
use tokio::sync::{
  mpsc,
  mpsc::{UnboundedReceiver, UnboundedSender},
};

pub enum SendActivityData {
  CreatePost { post: Post },
}

static ACTIVITY_CHANNEL: Lazy<(
  UnboundedSender<SendActivityData>,
  UnboundedReceiver<SendActivityData>,
)> = Lazy::new(|| mpsc::unbounded_channel());

pub fn activity_receiver() -> UnboundedReceiver<SendActivityData> {
  ACTIVITY_CHANNEL.deref().1
}

pub fn send_activity(data: SendActivityData) -> LemmyResult<()> {
  ACTIVITY_CHANNEL.deref().0.send(data)?;
  Ok(())
}
