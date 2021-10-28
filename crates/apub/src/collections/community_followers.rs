use crate::generate_followers_url;
use activitystreams::collection::kind::CollectionType;
use lemmy_api_common::blocking;
use lemmy_db_schema::source::community::Community;
use lemmy_db_views_actor::community_follower_view::CommunityFollowerView;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommunityFollowers {
  id: Url,
  r#type: CollectionType,
  total_items: i32,
  items: Vec<()>,
}

impl CommunityFollowers {
  pub(crate) async fn new(
    community: Community,
    context: &LemmyContext,
  ) -> Result<CommunityFollowers, LemmyError> {
    let community_id = community.id;
    let community_followers = blocking(context.pool(), move |conn| {
      CommunityFollowerView::for_community(conn, community_id)
    })
    .await??;

    Ok(CommunityFollowers {
      id: generate_followers_url(&community.actor_id)?.into_inner(),
      r#type: CollectionType::Collection,
      total_items: community_followers.len() as i32,
      items: vec![],
    })
  }
}
