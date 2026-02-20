use crate::{is_new_instance, protocol::collections::group_outbox::GroupOutbox};
use activitypub_federation::{
  config::Data,
  kinds::collection::OrderedCollectionType,
  protocol::verification::verify_domains_match,
  traits::{Activity, Collection},
};
use futures::future::join_all;
use lemmy_api_utils::{context::LemmyContext, utils::generate_outbox_url};
use lemmy_apub_activities::{
  activity_lists::AnnouncableActivities,
  protocol::{
    CreateOrUpdateType,
    community::announce::AnnounceActivity,
    create_or_update::page::CreateOrUpdatePage,
  },
};
use lemmy_apub_objects::objects::community::ApubCommunity;
use lemmy_db_schema::{source::site::Site, utils::FETCH_LIMIT_MAX};
use lemmy_db_schema_file::enums::PostSortType;
use lemmy_db_views_post::impls::PostQuery;
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

#[derive(Clone, Debug)]
pub(crate) struct ApubCommunityOutbox(());

#[async_trait::async_trait]
impl Collection for ApubCommunityOutbox {
  type Owner = ApubCommunity;
  type DataType = LemmyContext;
  type Kind = GroupOutbox;
  type Error = LemmyError;

  async fn read_local(owner: &Self::Owner, data: &Data<Self::DataType>) -> LemmyResult<Self::Kind> {
    let site = Site::read_local(&mut data.pool()).await?;

    let mut post_views = Box::pin(
      PostQuery {
        community_id: Some(owner.id),
        sort: Some(PostSortType::New),
        limit: Some(FETCH_LIMIT_MAX.try_into()?),
        ..Default::default()
      }
      .list(&site, &mut data.pool()),
    )
    .await?
    .items;

    // Outbox must be sorted reverse chronological (newest items first). This is already done
    // via SQL, but featured posts are always at the top so we need to manually sort it here.
    post_views.sort_unstable_by(|p1, p2| p2.post.published_at.cmp(&p1.post.published_at));

    let mut ordered_items = vec![];
    for post_view in post_views {
      // ignore errors, in particular if post creator was deleted
      if let Ok(create) = CreateOrUpdatePage::new(
        post_view.post.into(),
        &post_view.creator.into(),
        owner,
        CreateOrUpdateType::Create,
        data,
      )
      .await
      {
        let announcable = AnnouncableActivities::CreateOrUpdatePost(create);
        if let Ok(announce) = AnnounceActivity::new(announcable.try_into()?, owner, data) {
          ordered_items.push(announce);
        }
      }
    }

    Ok(GroupOutbox {
      r#type: OrderedCollectionType::OrderedCollection,
      id: generate_outbox_url(&owner.ap_id)?.into(),
      total_items: owner.posts,
      ordered_items,
    })
  }

  async fn verify(
    group_outbox: &GroupOutbox,
    expected_domain: &Url,
    _data: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    verify_domains_match(expected_domain, &group_outbox.id)?;
    Ok(())
  }

  async fn from_json(
    apub: Self::Kind,
    _owner: &Self::Owner,
    data: &Data<Self::DataType>,
  ) -> LemmyResult<Self> {
    // Fetch less posts on new instance to save requests
    let fetch_limit = if is_new_instance(data).await? {
      10
    } else {
      FETCH_LIMIT_MAX
    };
    let mut outbox_activities = apub.ordered_items;
    if outbox_activities.len() > fetch_limit {
      outbox_activities = outbox_activities
        .get(0..(fetch_limit))
        .unwrap_or_default()
        .to_vec();
    }

    // We intentionally ignore errors here. This is because the outbox might contain posts from old
    // Lemmy versions, or from other software which we cant parse. In that case, we simply skip the
    // item and only parse the ones that work.
    // process items in parallel, to avoid long delay from fetch_site_metadata() and other
    // processing
    join_all(outbox_activities.into_iter().map(|activity| {
      async {
        // Receiving announce requires at least one local community follower for anti spam purposes.
        // This won't be the case for newly fetched communities, so we extract the inner activity
        // and handle it directly to bypass this check.
        let inner = activity.object.object(data).await.map(TryInto::try_into);
        if let Ok(Ok(AnnouncableActivities::CreateOrUpdatePost(inner))) = inner {
          let verify = inner.verify(data).await;
          if verify.is_ok() {
            inner.receive(data).await.ok();
          }
        }
      }
    }))
    .await;

    // This return value is unused, so just set an empty vec
    Ok(ApubCommunityOutbox(()))
  }
}
