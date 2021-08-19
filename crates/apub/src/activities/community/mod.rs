use crate::{check_is_apub_id_valid, CommunityType};
use itertools::Itertools;
use lemmy_db_schema::source::community::Community;
use lemmy_utils::{settings::structs::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use url::Url;

pub mod add_mod;
pub mod announce;
pub mod block_user;
pub mod remove_mod;
pub mod undo_block_user;
pub mod update;

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
