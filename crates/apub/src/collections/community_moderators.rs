use crate::{
  collections::CommunityContext,
  fetcher::object_id::ObjectId,
  generate_moderators_url,
  objects::person::ApubPerson,
};
use activitystreams::{chrono::NaiveDateTime, collection::kind::OrderedCollectionType};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{traits::ApubObject, verify::verify_domains_match};
use lemmy_db_schema::{
  source::community::{CommunityModerator, CommunityModeratorForm},
  traits::Joinable,
};
use lemmy_db_views_actor::community_moderator_view::CommunityModeratorView;
use lemmy_utils::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupModerators {
  r#type: OrderedCollectionType,
  id: Url,
  ordered_items: Vec<ObjectId<ApubPerson>>,
}

#[derive(Clone, Debug)]
pub(crate) struct ApubCommunityModerators(pub(crate) Vec<CommunityModeratorView>);

#[async_trait::async_trait(?Send)]
impl ApubObject for ApubCommunityModerators {
  type DataType = CommunityContext;
  type TombstoneType = ();
  type ApubType = GroupModerators;

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    None
  }

  async fn read_from_apub_id(
    _object_id: Url,
    data: &Self::DataType,
  ) -> Result<Option<Self>, LemmyError> {
    // Only read from database if its a local community, otherwise fetch over http
    if data.0.local {
      let cid = data.0.id;
      let moderators = blocking(data.1.pool(), move |conn| {
        CommunityModeratorView::for_community(conn, cid)
      })
      .await??;
      Ok(Some(ApubCommunityModerators { 0: moderators }))
    } else {
      Ok(None)
    }
  }

  async fn delete(self, _data: &Self::DataType) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn to_apub(&self, data: &Self::DataType) -> Result<Self::ApubType, LemmyError> {
    let ordered_items = self
      .0
      .iter()
      .map(|m| ObjectId::<ApubPerson>::new(m.moderator.actor_id.clone().into_inner()))
      .collect();
    Ok(GroupModerators {
      r#type: OrderedCollectionType::OrderedCollection,
      id: generate_moderators_url(&data.0.actor_id)?.into(),
      ordered_items,
    })
  }

  fn to_tombstone(&self) -> Result<Self::TombstoneType, LemmyError> {
    unimplemented!()
  }

  async fn from_apub(
    apub: &Self::ApubType,
    data: &Self::DataType,
    expected_domain: &Url,
    request_counter: &mut i32,
  ) -> Result<Self, LemmyError> {
    verify_domains_match(expected_domain, &apub.id)?;
    let community_id = data.0.id;
    let current_moderators = blocking(data.1.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await??;
    // Remove old mods from database which arent in the moderators collection anymore
    for mod_user in &current_moderators {
      let mod_id = ObjectId::new(mod_user.moderator.actor_id.clone().into_inner());
      if !apub.ordered_items.contains(&mod_id) {
        let community_moderator_form = CommunityModeratorForm {
          community_id: mod_user.community.id,
          person_id: mod_user.moderator.id,
        };
        blocking(data.1.pool(), move |conn| {
          CommunityModerator::leave(conn, &community_moderator_form)
        })
        .await??;
      }
    }

    // Add new mods to database which have been added to moderators collection
    for mod_id in &apub.ordered_items {
      let mod_id = ObjectId::new(mod_id.clone());
      let mod_user: ApubPerson = mod_id.dereference(&data.1, request_counter).await?;

      if !current_moderators
        .clone()
        .iter()
        .map(|c| c.moderator.actor_id.clone())
        .any(|x| x == mod_user.actor_id)
      {
        let community_moderator_form = CommunityModeratorForm {
          community_id: data.0.id,
          person_id: mod_user.id,
        };
        blocking(data.1.pool(), move |conn| {
          CommunityModerator::join(conn, &community_moderator_form)
        })
        .await??;
      }
    }

    // This return value is unused, so just set an empty vec
    Ok(ApubCommunityModerators { 0: vec![] })
  }
}
