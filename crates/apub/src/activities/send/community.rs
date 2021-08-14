use crate::{check_is_apub_id_valid, ActorType, CommunityType};
use itertools::Itertools;
use lemmy_api_common::blocking;
use lemmy_db_queries::DbPool;
use lemmy_db_schema::source::community::Community;
use lemmy_db_views_actor::community_follower_view::CommunityFollowerView;
use lemmy_utils::LemmyError;
use url::Url;

impl ActorType for Community {
  fn is_local(&self) -> bool {
    self.local
  }
  fn actor_id(&self) -> Url {
    self.actor_id.to_owned().into()
  }
  fn name(&self) -> String {
    self.name.clone()
  }
  fn public_key(&self) -> Option<String> {
    self.public_key.to_owned()
  }
  fn private_key(&self) -> Option<String> {
    self.private_key.to_owned()
  }

  fn get_shared_inbox_or_inbox_url(&self) -> Url {
    self
      .shared_inbox_url
      .clone()
      .unwrap_or_else(|| self.inbox_url.to_owned())
      .into()
  }
}

#[async_trait::async_trait(?Send)]
impl CommunityType for Community {
  fn followers_url(&self) -> Url {
    self.followers_url.clone().into()
  }

  /// For a given community, returns the inboxes of all followers.
  async fn get_follower_inboxes(&self, pool: &DbPool) -> Result<Vec<Url>, LemmyError> {
    let id = self.id;

    let follows = blocking(pool, move |conn| {
      CommunityFollowerView::for_community(conn, id)
    })
    .await??;
    let inboxes = follows
      .into_iter()
      .filter(|f| !f.follower.local)
      .map(|f| f.follower.shared_inbox_url.unwrap_or(f.follower.inbox_url))
      .map(|i| i.into_inner())
      .unique()
      // Don't send to blocked instances
      .filter(|inbox| check_is_apub_id_valid(inbox, false).is_ok())
      .collect();

    Ok(inboxes)
  }
}
