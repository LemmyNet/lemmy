use actix_web::{web, web::Data};
use lemmy_api_common::{comment::*, community::*, person::*, post::*, private_message::*, site::*};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::{serialize_websocket_message, LemmyContext, UserOperationCrud};
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

pub async fn match_websocket_operation_crud(
  context: LemmyContext,
  id: ConnectionId,
  op: UserOperationCrud,
  data: &str,
) -> Result<String, LemmyError> {
  //TODO: handle commented out actions in crud crate

  match op {
    // User ops
    UserOperationCrud::Register => do_websocket_operation::<Register>(context, id, op, data).await,
    UserOperationCrud::GetPersonDetails => {
      do_websocket_operation::<GetPersonDetails>(context, id, op, data).await
    }
    UserOperationCrud::DeleteAccount => {
      do_websocket_operation::<DeleteAccount>(context, id, op, data).await
    }

    // Private Message ops
    UserOperationCrud::CreatePrivateMessage => {
      do_websocket_operation::<CreatePrivateMessage>(context, id, op, data).await
    }
    UserOperationCrud::EditPrivateMessage => {
      do_websocket_operation::<EditPrivateMessage>(context, id, op, data).await
    }
    UserOperationCrud::DeletePrivateMessage => {
      do_websocket_operation::<DeletePrivateMessage>(context, id, op, data).await
    }
    UserOperationCrud::GetPrivateMessages => {
      do_websocket_operation::<GetPrivateMessages>(context, id, op, data).await
    }

    // Site ops
    UserOperationCrud::CreateSite => {
      do_websocket_operation::<CreateSite>(context, id, op, data).await
    }
    UserOperationCrud::EditSite => do_websocket_operation::<EditSite>(context, id, op, data).await,
    UserOperationCrud::GetSite => do_websocket_operation::<GetSite>(context, id, op, data).await,

    // Community ops
    UserOperationCrud::GetCommunity => {
      do_websocket_operation::<GetCommunity>(context, id, op, data).await
    }
    UserOperationCrud::ListCommunities => {
      do_websocket_operation::<ListCommunities>(context, id, op, data).await
    }
    UserOperationCrud::CreateCommunity => {
      do_websocket_operation::<CreateCommunity>(context, id, op, data).await
    }
    UserOperationCrud::EditCommunity => {
      do_websocket_operation::<EditCommunity>(context, id, op, data).await
    }
    UserOperationCrud::DeleteCommunity => {
      do_websocket_operation::<DeleteCommunity>(context, id, op, data).await
    }
    UserOperationCrud::RemoveCommunity => {
      do_websocket_operation::<RemoveCommunity>(context, id, op, data).await
    }

    // Post ops
    UserOperationCrud::CreatePost => {
      do_websocket_operation::<CreatePost>(context, id, op, data).await
    }
    UserOperationCrud::GetPost => do_websocket_operation::<GetPost>(context, id, op, data).await,
    UserOperationCrud::GetPosts => do_websocket_operation::<GetPosts>(context, id, op, data).await,
    UserOperationCrud::EditPost => do_websocket_operation::<EditPost>(context, id, op, data).await,
    UserOperationCrud::DeletePost => {
      do_websocket_operation::<DeletePost>(context, id, op, data).await
    }
    UserOperationCrud::RemovePost => {
      do_websocket_operation::<RemovePost>(context, id, op, data).await
    }

    // Comment ops
    UserOperationCrud::CreateComment => {
      do_websocket_operation::<CreateComment>(context, id, op, data).await
    }
    UserOperationCrud::EditComment => {
      do_websocket_operation::<EditComment>(context, id, op, data).await
    }
    UserOperationCrud::DeleteComment => {
      do_websocket_operation::<DeleteComment>(context, id, op, data).await
    }
    UserOperationCrud::RemoveComment => {
      do_websocket_operation::<RemoveComment>(context, id, op, data).await
    }
    UserOperationCrud::GetComment => {
      do_websocket_operation::<GetComment>(context, id, op, data).await
    }
    UserOperationCrud::GetComments => {
      do_websocket_operation::<GetComments>(context, id, op, data).await
    }
  }
}

async fn do_websocket_operation<'a, 'b, Data>(
  context: LemmyContext,
  id: ConnectionId,
  op: UserOperationCrud,
  data: &str,
) -> Result<String, LemmyError>
where
  for<'de> Data: Deserialize<'de> + 'a,
  Data: PerformCrud,
{
  let parsed_data: Data = serde_json::from_str(data)?;
  let res = parsed_data
    .perform(&web::Data::new(context), Some(id))
    .await?;
  serialize_websocket_message(&op, &res)
}
