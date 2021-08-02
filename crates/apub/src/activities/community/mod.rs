use crate::{check_is_apub_id_valid, CommunityType};
use itertools::Itertools;
use lemmy_api_common::{blocking, community::CommunityResponse};
use lemmy_db_schema::{source::community::Community, CommunityId};
use lemmy_db_views_actor::community_view::CommunityView;
use lemmy_utils::{settings::structs::Settings, LemmyError};
use lemmy_websocket::{messages::SendCommunityRoomMessage, LemmyContext};
use url::Url;

pub mod add_mod;
pub mod announce;
pub mod block_user;
pub mod undo_block_user;
pub mod update;

pub(crate) async fn send_websocket_message<
  OP: ToString + Send + lemmy_websocket::OperationType + 'static,
>(
  community_id: CommunityId,
  op: OP,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let community_view = blocking(context.pool(), move |conn| {
    CommunityView::read(conn, community_id, None)
  })
  .await??;

  let res = CommunityResponse { community_view };

  context.chat_server().do_send(SendCommunityRoomMessage {
    op,
    response: res,
    community_id,
    websocket_id: None,
  });

  Ok(())
}

async fn list_community_follower_inboxes(
  community: &Community,
  additional_inboxes: Vec<Url>,
  context: &LemmyContext,
) -> Result<Vec<Url>, LemmyError> {
  Ok(
    vec![
      community.get_follower_inboxes(context.pool()).await?,
      additional_inboxes,
    ]
    .iter()
    .flatten()
    .unique()
    .filter(|inbox| inbox.host_str() != Some(&Settings::get().hostname))
    .filter(|inbox| check_is_apub_id_valid(inbox, false).is_ok())
    .map(|inbox| inbox.to_owned())
    .collect(),
  )
}
