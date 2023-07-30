use actix_web::web::Data;
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyError;

pub mod comment;
pub mod community;
pub mod custom_emoji;
pub mod post;
pub mod private_message;
pub mod site;
pub mod user;

#[async_trait::async_trait(?Send)]
pub trait PerformCrud {
  type Response: serde::ser::Serialize + Send + Clone + Sync;

  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError>;
}
