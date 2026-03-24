use crate::{objects::person::ApubPerson, protocol::feed_moderators::FeedModerators};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::collection::OrderedCollectionType,
  protocol::verification::verify_domains_match,
  traits::Collection,
};
use lemmy_api_utils::{context::LemmyContext, utils::generate_moderators_url};
use lemmy_db_schema::source::{multi_community::MultiCommunity, person::Person};
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubFeedModerators(pub ApubPerson);

#[async_trait::async_trait]
impl Collection for ApubFeedModerators {
  type Owner = Option<MultiCommunity>;
  type DataType = LemmyContext;
  type Kind = FeedModerators;
  type Error = LemmyError;

  async fn read_local(owner: &Self::Owner, data: &Data<Self::DataType>) -> LemmyResult<Self::Kind> {
    let owner = owner.as_ref().unwrap();
    let moderator = Person::read(&mut data.pool(), owner.creator_id).await?;

    let ordered_items = vec![ObjectId::<ApubPerson>::from(moderator.ap_id)];
    Ok(FeedModerators {
      r#type: OrderedCollectionType::OrderedCollection,
      id: generate_moderators_url(&owner.ap_id)?.into(),
      ordered_items,
    })
  }

  async fn verify(
    feed_moderators: &FeedModerators,
    expected_domain: &Url,
    _data: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    verify_domains_match(&feed_moderators.id, expected_domain)?;
    Ok(())
  }

  async fn from_json(
    apub: Self::Kind,
    _owner: &Self::Owner,
    data: &Data<Self::DataType>,
  ) -> LemmyResult<Self> {
    let moderator = apub
      .ordered_items
      .first()
      .unwrap()
      .dereference(data)
      .await?;
    Ok(Self(moderator))
  }
}
