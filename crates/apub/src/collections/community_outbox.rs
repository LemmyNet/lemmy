use crate::{
  activity_lists::AnnouncableActivities,
  collections::CommunityContext,
  objects::post::ApubPost,
  protocol::{
    activities::{
      community::announce::AnnounceActivity,
      create_or_update::page::CreateOrUpdatePage,
      CreateOrUpdateType,
    },
    collections::group_outbox::GroupOutbox,
  },
};
use activitypub_federation::{
  data::Data,
  traits::{ActivityHandler, ApubObject},
  utils::verify_domains_match,
};
use activitystreams_kinds::collection::OrderedCollectionType;
use chrono::NaiveDateTime;
use futures::future::join_all;
use lemmy_api_common::utils::generate_outbox_url;
use lemmy_db_schema::{
  source::{person::Person, post::Post},
  traits::Crud,
  utils::FETCH_LIMIT_MAX,
};
use lemmy_utils::error::LemmyError;
use url::Url;

#[derive(Clone, Debug)]
pub(crate) struct ApubCommunityOutbox(Vec<ApubPost>);

#[async_trait::async_trait(?Send)]
impl ApubObject for ApubCommunityOutbox {
  type DataType = CommunityContext;
  type ApubType = GroupOutbox;
  type Error = LemmyError;
  type DbType = ();

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    None
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_apub_id(
    _object_id: Url,
    data: &Self::DataType,
  ) -> Result<Option<Self>, LemmyError> {
    // Only read from database if its a local community, otherwise fetch over http
    if data.0.local {
      let community_id = data.0.id;
      let post_list: Vec<ApubPost> = Post::list_for_community(data.1.pool(), community_id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
      Ok(Some(ApubCommunityOutbox(post_list)))
    } else {
      Ok(None)
    }
  }

  #[tracing::instrument(skip_all)]
  async fn into_apub(self, data: &Self::DataType) -> Result<Self::ApubType, LemmyError> {
    let mut ordered_items = vec![];
    for post in self.0 {
      let person = Person::read(data.1.pool(), post.creator_id).await?.into();
      let create =
        CreateOrUpdatePage::new(post, &person, &data.0, CreateOrUpdateType::Create, &data.1)
          .await?;
      let announcable = AnnouncableActivities::CreateOrUpdatePost(create);
      let announce = AnnounceActivity::new(announcable.try_into()?, &data.0, &data.1)?;
      ordered_items.push(announce);
    }

    Ok(GroupOutbox {
      r#type: OrderedCollectionType::OrderedCollection,
      id: generate_outbox_url(&data.0.actor_id)?.into(),
      total_items: ordered_items.len() as i32,
      ordered_items,
    })
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    group_outbox: &GroupOutbox,
    expected_domain: &Url,
    _context: &CommunityContext,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_domains_match(expected_domain, &group_outbox.id)?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn from_apub(
    apub: Self::ApubType,
    data: &Self::DataType,
    _request_counter: &mut i32,
  ) -> Result<Self, LemmyError> {
    let mut outbox_activities = apub.ordered_items;
    if outbox_activities.len() as i64 > FETCH_LIMIT_MAX {
      outbox_activities = outbox_activities
        .get(0..(FETCH_LIMIT_MAX as usize))
        .unwrap_or_default()
        .to_vec();
    }

    // We intentionally ignore errors here. This is because the outbox might contain posts from old
    // Lemmy versions, or from other software which we cant parse. In that case, we simply skip the
    // item and only parse the ones that work.
    let data = Data::new(data.1.clone());
    // process items in parallel, to avoid long delay from fetch_site_metadata() and other processing
    join_all(outbox_activities.into_iter().map(|activity| {
      async {
        // use separate request counter for each item, otherwise there will be problems with
        // parallel processing
        let request_counter = &mut 0;
        let verify = activity.verify(&data, request_counter).await;
        if verify.is_ok() {
          activity.receive(&data, request_counter).await.ok();
        }
      }
    }))
    .await;

    // This return value is unused, so just set an empty vec
    Ok(ApubCommunityOutbox(Vec::new()))
  }
}
