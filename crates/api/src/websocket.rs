use crate::{get_user_from_jwt, Perform};
use actix_web::web::Data;
use lemmy_structs::websocket::*;
use lemmy_utils::{ConnectionId, LemmyError};
use lemmy_websocket::{
  messages::{JoinCommunityRoom, JoinModRoom, JoinPostRoom, JoinUserRoom},
  LemmyContext,
};

#[async_trait::async_trait(?Send)]
impl Perform for UserJoin {
  type Response = UserJoinResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<UserJoinResponse, LemmyError> {
    let data: &UserJoin = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    if let Some(ws_id) = websocket_id {
      context.chat_server().do_send(JoinUserRoom {
        user_id: user.id,
        id: ws_id,
      });
    }

    Ok(UserJoinResponse { joined: true })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for CommunityJoin {
  type Response = CommunityJoinResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityJoinResponse, LemmyError> {
    let data: &CommunityJoin = &self;

    if let Some(ws_id) = websocket_id {
      context.chat_server().do_send(JoinCommunityRoom {
        community_id: data.community_id,
        id: ws_id,
      });
    }

    Ok(CommunityJoinResponse { joined: true })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for ModJoin {
  type Response = ModJoinResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<ModJoinResponse, LemmyError> {
    let data: &ModJoin = &self;

    if let Some(ws_id) = websocket_id {
      context.chat_server().do_send(JoinModRoom {
        community_id: data.community_id,
        id: ws_id,
      });
    }

    Ok(ModJoinResponse { joined: true })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for PostJoin {
  type Response = PostJoinResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostJoinResponse, LemmyError> {
    let data: &PostJoin = &self;

    if let Some(ws_id) = websocket_id {
      context.chat_server().do_send(JoinPostRoom {
        post_id: data.post_id,
        id: ws_id,
      });
    }

    Ok(PostJoinResponse { joined: true })
  }
}
