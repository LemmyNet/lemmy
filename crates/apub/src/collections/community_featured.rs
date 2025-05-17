use crate::{
  objects::{community::ApubCommunity, post::ApubPost},
  protocol::collections::group_featured::GroupFeatured,
};
use activitypub_federation::{
  config::Data,
  kinds::collection::OrderedCollectionType,
  protocol::verification::verify_domains_match,
  traits::{Collection, Object},
};
use futures::future::{join_all, try_join_all};
use lemmy_api_common::{context::LemmyContext, utils::generate_featured_url};
use lemmy_db_schema::{
  source::{community::Community, post::Post},
  utils::FETCH_LIMIT_MAX,
};
use lemmy_utils::error::LemmyError;
use url::Url;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ApubCommunityFeatured(());

#[async_trait::async_trait]
impl Collection for ApubCommunityFeatured {
  type Owner = ApubCommunity;
  type DataType = LemmyContext;
  type Kind = GroupFeatured;
  type Error = LemmyError;

  async fn read_local(
    owner: &Self::Owner,
    data: &Data<Self::DataType>,
  ) -> Result<Self::Kind, Self::Error> {
    let ordered_items = try_join_all(
      Post::list_featured_for_community(&mut data.pool(), owner.id)
        .await?
        .into_iter()
        .map(ApubPost::from)
        .map(|p| p.into_json(data)),
    )
    .await?;
    Ok(GroupFeatured {
      r#type: OrderedCollectionType::OrderedCollection,
      id: generate_featured_url(&owner.actor_id)?.into(),
      total_items: ordered_items.len() as i32,
      ordered_items,
    })
  }

  async fn verify(
    apub: &Self::Kind,
    expected_domain: &Url,
    _data: &Data<Self::DataType>,
  ) -> Result<(), Self::Error> {
    verify_domains_match(expected_domain, &apub.id)?;
    Ok(())
  }

  async fn from_json(
    apub: Self::Kind,
    owner: &Self::Owner,
    context: &Data<Self::DataType>,
  ) -> Result<Self, Self::Error>
  where
    Self: Sized,
  {
    let mut pages = apub.ordered_items;
    if pages.len() as i64 > FETCH_LIMIT_MAX {
      pages = pages
        .get(0..(FETCH_LIMIT_MAX as usize))
        .unwrap_or_default()
        .to_vec();
    }

    // process items in parallel, to avoid long delay from fetch_site_metadata() and other
    // processing
    let stickied_posts: Vec<Post> = join_all(pages.into_iter().map(|page| {
      async {
        // use separate request counter for each item, otherwise there will be problems with
        // parallel processing
        ApubPost::verify(&page, &apub.id, context).await?;
        ApubPost::from_json(page, context).await
      }
    }))
    .await
    // ignore any failed or unparseable items
    .into_iter()
    .filter_map(|p| p.ok().map(|p| p.0))
    .collect();

    Community::set_featured_posts(owner.id, stickied_posts, &mut context.pool()).await?;

    // This return value is unused, so just set an empty vec
    Ok(ApubCommunityFeatured(()))
  }
}
