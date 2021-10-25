use crate::{
  activities::{post::create_or_update::CreateOrUpdatePost, CreateOrUpdateType},
  context::lemmy_context,
  generate_outbox_url,
  objects::{community::ApubCommunity, person::ApubPerson, post::ApubPost},
};
use activitystreams::{
  base::AnyBase,
  chrono::NaiveDateTime,
  collection::kind::OrderedCollectionType,
  object::Tombstone,
  primitives::OneOrMany,
  url::Url,
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  data::Data,
  traits::{ActivityHandler, ApubObject},
  verify::verify_domains_match,
};
use lemmy_db_schema::{
  source::{person::Person, post::Post},
  traits::Crud,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityOutbox {
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  r#type: OrderedCollectionType,
  id: Url,
  ordered_items: Vec<CreateOrUpdatePost>,
}

#[derive(Clone, Debug)]
pub(crate) struct ApubCommunityOutbox(Vec<ApubPost>);

/// Put community in the data, so we dont have to read it again from the database.
pub(crate) struct OutboxData(pub ApubCommunity, pub LemmyContext);

#[async_trait::async_trait(?Send)]
impl ApubObject for ApubCommunityOutbox {
  type DataType = OutboxData;
  type TombstoneType = Tombstone;
  type ApubType = CommunityOutbox;

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    None
  }

  async fn read_from_apub_id(
    _object_id: Url,
    data: &Self::DataType,
  ) -> Result<Option<Self>, LemmyError> {
    // Only read from database if its a local community, otherwise fetch over http
    if data.0.local {
      let community_id = data.0.id;
      let post_list: Vec<ApubPost> = blocking(data.1.pool(), move |conn| {
        Post::list_for_community(conn, community_id)
      })
      .await??
      .into_iter()
      .map(Into::into)
      .collect();
      Ok(Some(ApubCommunityOutbox(post_list)))
    } else {
      Ok(None)
    }
  }

  async fn delete(self, _data: &Self::DataType) -> Result<(), LemmyError> {
    // do nothing (it gets deleted automatically with the community)
    Ok(())
  }

  async fn to_apub(&self, data: &Self::DataType) -> Result<Self::ApubType, LemmyError> {
    let mut ordered_items = vec![];
    for post in &self.0 {
      let actor = post.creator_id;
      let actor: ApubPerson = blocking(data.1.pool(), move |conn| Person::read(conn, actor))
        .await??
        .into();
      let a =
        CreateOrUpdatePost::new(post, &actor, &data.0, CreateOrUpdateType::Create, &data.1).await?;
      ordered_items.push(a);
    }

    Ok(CommunityOutbox {
      context: lemmy_context(),
      r#type: OrderedCollectionType::OrderedCollection,
      id: generate_outbox_url(&data.0.actor_id)?.into(),
      ordered_items,
    })
  }

  fn to_tombstone(&self) -> Result<Self::TombstoneType, LemmyError> {
    // no tombstone for this, there is only a tombstone for the community
    unimplemented!()
  }

  async fn from_apub(
    apub: &Self::ApubType,
    data: &Self::DataType,
    expected_domain: &Url,
    request_counter: &mut i32,
  ) -> Result<Self, LemmyError> {
    verify_domains_match(expected_domain, &apub.id)?;
    let mut outbox_activities = apub.ordered_items.clone();
    if outbox_activities.len() > 20 {
      outbox_activities = outbox_activities[0..20].to_vec();
    }

    // We intentionally ignore errors here. This is because the outbox might contain posts from old
    // Lemmy versions, or from other software which we cant parse. In that case, we simply skip the
    // item and only parse the ones that work.
    for activity in outbox_activities {
      activity
        .receive(&Data::new(data.1.clone()), request_counter)
        .await
        .ok();
    }

    // This return value is unused, so just set an empty vec
    Ok(ApubCommunityOutbox { 0: vec![] })
  }
}
