use super::multi_community::ApubMultiCommunity;
use crate::protocol::multi_community::FeedCollection;
use activitypub_federation::{
  config::Data,
  protocol::verification::verify_domains_match,
  traits::Collection,
};
use futures::future::join_all;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::{
    community::{CommunityActions, CommunityFollowerForm},
    multi_community::MultiCommunity,
  },
  traits::Followable,
};
use lemmy_db_schema_file::enums::CommunityFollowerState;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::{LemmyError, LemmyResult};
use tracing::info;
use url::Url;

pub struct ApubFeedCollection;

#[async_trait::async_trait]
impl Collection for ApubFeedCollection {
  type DataType = LemmyContext;
  type Kind = FeedCollection;
  type Owner = ApubMultiCommunity;
  type Error = LemmyError;

  async fn read_local(
    owner: &Self::Owner,
    context: &Data<Self::DataType>,
  ) -> Result<Self::Kind, Self::Error> {
    let entries = MultiCommunity::read_entry_ap_ids(&mut context.pool(), &owner.name).await?;
    Ok(Self::Kind {
      r#type: Default::default(),
      id: owner.following_url.clone().into(),
      total_items: entries.len().try_into()?,
      items: entries.into_iter().map(Into::into).collect(),
    })
  }

  async fn verify(
    json: &Self::Kind,
    expected_domain: &Url,
    _context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    verify_domains_match(expected_domain, &json.id.clone().into())?;
    Ok(())
  }

  async fn from_json(
    json: Self::Kind,
    owner: &Self::Owner,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<Self> {
    let communities = join_all(
      json
        .items
        .into_iter()
        .map(|ap_id| async move { Ok(ap_id.dereference(context).await?.id) }),
    )
    .await
    .into_iter()
    .flat_map(|c: LemmyResult<CommunityId>| match c {
      Ok(c) => Some(c),
      Err(e) => {
        info!("Failed to fetch multi-community item: {e}");
        None
      }
    })
    .collect();

    let (remote_added, remote_removed, has_local_followers) =
      MultiCommunity::update_entries(&mut context.pool(), owner.id, &communities).await?;

    // Have multi-comm follower bot follow all communities which were added to multi-comm,
    // and unfollow those that were removed.
    // If the multi-comm has no local followers its ignored.
    // TODO: This means there will be posts missing in multi-comm without local followers.
    if has_local_followers {
      let multicomm_follower = SiteView::read_multicomm_follower(&mut context.pool()).await?;
      for community in remote_added {
        let form = CommunityFollowerForm::new(
          community.id,
          multicomm_follower.id,
          CommunityFollowerState::Pending,
        );
        CommunityActions::follow(&mut context.pool(), &form).await?;
        ActivityChannel::submit_activity(
          SendActivityData::FollowCommunity(community.clone(), multicomm_follower.clone(), true),
          context,
        )?;
      }
      for community in remote_removed {
        CommunityActions::unfollow(&mut context.pool(), multicomm_follower.id, community.id)
          .await?;
        ActivityChannel::submit_activity(
          SendActivityData::FollowCommunity(community.clone(), multicomm_follower.clone(), false),
          context,
        )?;
      }
    }

    Ok(ApubFeedCollection)
  }
}
