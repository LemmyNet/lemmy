use actix_web::{web, web::Data};
use lemmy_api_common::{
  comment::{CreateComment, DeleteComment, EditComment, GetComment, GetComments, RemoveComment},
  community::{
    CreateCommunity,
    DeleteCommunity,
    EditCommunity,
    GetCommunity,
    ListCommunities,
    RemoveCommunity,
  },
  person::{DeleteAccount, GetPersonDetails, Register},
  post::{CreatePost, DeletePost, EditPost, GetPost, GetPosts, RemovePost},
  private_message::{
    CreatePrivateMessage,
    DeletePrivateMessage,
    EditPrivateMessage,
    GetPrivateMessages,
  },
  site::{CreateSite, EditSite, GetSite},
  websocket::{serialize_websocket_message, UserOperationCrud},
  LemmyContext,
};
use lemmy_utils::{error::LemmyError, ConnectionId};
use serde::Deserialize;

mod comment;
mod community;
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
