use activitypub_federation::kinds::collection::CollectionType;
use lemmy_api_common::{context::LemmyContext, utils::generate_followers_url};
use lemmy_db_schema::source::community::Community;
use lemmy_db_views_actor::structs::CommunityFollowerView;
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GroupFollowers {
  id: Url,
  r#type: CollectionType,
  total_items: i32,
  items: Vec<()>,
}

impl GroupFollowers {
  pub(crate) async fn new(
    community: Community,
    context: &LemmyContext,
  ) -> Result<GroupFollowers, LemmyError> {
    let community_id = community.id;
    let community_followers =
      CommunityFollowerView::count_community_followers(&mut context.pool(), community_id).await?;

    Ok(GroupFollowers {
      id: generate_followers_url(&community.actor_id)?.into(),
      r#type: CollectionType::Collection,
      total_items: community_followers as i32,
      items: vec![],
    })
  }
}
