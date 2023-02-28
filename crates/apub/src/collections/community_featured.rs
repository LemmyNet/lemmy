use crate::{
  collections::CommunityContext,
  objects::post::ApubPost,
  protocol::collections::group_featured::GroupFeatured,
};
use activitypub_federation::{
  data::Data,
  traits::{ActivityHandler, ApubObject},
  utils::verify_domains_match,
};
use activitystreams_kinds::collection::OrderedCollectionType;
use futures::future::{join_all, try_join_all};
use lemmy_api_common::utils::generate_featured_url;
use lemmy_db_schema::{source::post::Post, utils::FETCH_LIMIT_MAX};
use lemmy_utils::error::LemmyError;
use url::Url;

#[derive(Clone, Debug)]
pub(crate) struct ApubCommunityFeatured(Vec<ApubPost>);

#[async_trait::async_trait(?Send)]
impl ApubObject for ApubCommunityFeatured {
  type DataType = CommunityContext;
  type ApubType = GroupFeatured;
  type DbType = ();
  type Error = LemmyError;

  async fn read_from_apub_id(
    _object_id: Url,
    data: &Self::DataType,
  ) -> Result<Option<Self>, Self::Error>
  where
    Self: Sized,
  {
    // Only read from database if its a local community, otherwise fetch over http
    if data.0.local {
      let community_id = data.0.id;
      let post_list: Vec<ApubPost> = Post::list_featured_for_community(data.1.pool(), community_id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
      Ok(Some(ApubCommunityFeatured(post_list)))
    } else {
      Ok(None)
    }
  }

  async fn into_apub(self, data: &Self::DataType) -> Result<Self::ApubType, Self::Error> {
    let ordered_items = try_join_all(self.0.into_iter().map(|p| p.into_apub(&data.1))).await?;
    Ok(GroupFeatured {
      r#type: OrderedCollectionType::OrderedCollection,
      id: generate_featured_url(&data.0.actor_id)?.into(),
      total_items: ordered_items.len() as i32,
      ordered_items,
    })
  }

  async fn verify(
    apub: &Self::ApubType,
    expected_domain: &Url,
    _data: &Self::DataType,
    _request_counter: &mut i32,
  ) -> Result<(), Self::Error> {
    verify_domains_match(expected_domain, &apub.id)?;
    Ok(())
  }

  async fn from_apub(
    apub: Self::ApubType,
    data: &Self::DataType,
    _request_counter: &mut i32,
  ) -> Result<Self, Self::Error>
  where
    Self: Sized,
  {
    let mut posts = apub.ordered_items;
    if posts.len() as i64 > FETCH_LIMIT_MAX {
      posts = posts
        .get(0..(FETCH_LIMIT_MAX as usize))
        .unwrap_or_default()
        .to_vec();
    }

    // We intentionally ignore errors here. This is because the outbox might contain posts from old
    // Lemmy versions, or from other software which we cant parse. In that case, we simply skip the
    // item and only parse the ones that work.
    let data = Data::new(data.1.clone());
    // process items in parallel, to avoid long delay from fetch_site_metadata() and other processing
    join_all(posts.into_iter().map(|post| {
      async {
        // use separate request counter for each item, otherwise there will be problems with
        // parallel processing
        let request_counter = &mut 0;
        let verify = post.verify(&data, request_counter).await;
        if verify.is_ok() {
          post.receive(&data, request_counter).await.ok();
        }
      }
    }))
    .await;

    // This return value is unused, so just set an empty vec
    Ok(ApubCommunityFeatured(Vec::new()))
  }
}
