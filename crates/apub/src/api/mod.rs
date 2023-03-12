use activitypub_federation::config::Data;
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::{error::LemmyError, ConnectionId};

mod list_comments;
mod list_posts;
mod read_community;
mod read_person;
mod resolve_object;
mod search;

#[async_trait::async_trait]
pub trait PerformApub {
  type Response: serde::ser::Serialize + Send;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError>;
}
