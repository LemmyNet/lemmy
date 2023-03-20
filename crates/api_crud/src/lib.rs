use actix_web::web::Data;
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::{error::LemmyError, ConnectionId};

mod comment;
mod community;
mod custom_emoji;
mod post;
mod private_message;
mod site;
mod user;

#[async_trait::async_trait(?Send)]
pub trait PerformCrud {
  type Response: serde::ser::Serialize + Send;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError>;
}
