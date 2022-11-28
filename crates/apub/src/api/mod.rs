use actix_web::web::Data;
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::{error::LemmyError, ConnectionId};

mod list_comments;
mod list_posts;
mod read_community;
mod read_person;
mod resolve_object;
mod search;

#[async_trait::async_trait(?Send)]
pub trait PerformApub {
  type Response: serde::ser::Serialize + Send;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError>;
}
