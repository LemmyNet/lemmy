use actix_web::web::Data;
use lemmy_utils::{ConnectionId, LemmyError};
use lemmy_websocket::LemmyContext;

mod comment;
mod community;
mod post;
mod private_message;
pub mod routes;
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
