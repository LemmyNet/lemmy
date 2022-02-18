use crate::{
  collections::CommunityContext,
  generate_moderators_url,
  objects::person::ApubPerson,
  protocol::collections::group_moderators::GroupModerators,
};
use activitystreams_kinds::collection::OrderedCollectionType;
use chrono::NaiveDateTime;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{object_id::ObjectId, traits::ApubObject, verify::verify_domains_match};
use lemmy_db_schema::{
  source::community::{CommunityModerator, CommunityModeratorForm},
  traits::Joinable,
};
use lemmy_db_views_actor::community_moderator_view::CommunityModeratorView;
use lemmy_utils::LemmyError;
use url::Url;

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

  #[tracing::instrument(skip_all)]
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

  #[tracing::instrument(skip_all)]
  async fn delete(self, _data: &Self::DataType) -> Result<(), LemmyError> {
    unimplemented!()
  }

  #[tracing::instrument(skip_all)]
  async fn into_apub(self, data: &Self::DataType) -> Result<Self::ApubType, LemmyError> {
    let ordered_items = self
      .0
      .into_iter()
      .map(|m| ObjectId::<ApubPerson>::new(m.moderator.actor_id))
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

  #[tracing::instrument(skip_all)]
  async fn verify(
    group_moderators: &GroupModerators,
    expected_domain: &Url,
    _context: &CommunityContext,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_domains_match(&group_moderators.id, expected_domain)?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn from_apub(
    apub: Self::ApubType,
    data: &Self::DataType,
    request_counter: &mut i32,
  ) -> Result<Self, LemmyError> {
    let community_id = data.0.id;
    let current_moderators = blocking(data.1.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await??;
    // Remove old mods from database which arent in the moderators collection anymore
    for mod_user in &current_moderators {
      let mod_id = ObjectId::new(mod_user.moderator.actor_id.clone());
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
    for mod_id in apub.ordered_items {
      let mod_id = ObjectId::new(mod_id);
      let mod_user: ApubPerson = mod_id
        .dereference(&data.1, data.1.client(), request_counter)
        .await?;

      if !current_moderators
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    objects::{
      community::tests::parse_lemmy_community,
      person::tests::parse_lemmy_person,
      tests::init_context,
    },
    protocol::tests::file_to_json_object,
  };
  use lemmy_apub_lib::activity_queue::create_activity_queue;
  use lemmy_db_schema::{
    source::{
      community::Community,
      person::{Person, PersonForm},
      site::Site,
    },
    traits::Crud,
  };
  use serial_test::serial;

  #[actix_rt::test]
  #[serial]
  async fn test_parse_lemmy_community_moderators() {
    let client = reqwest::Client::new().into();
    let manager = create_activity_queue(client);
    let context = init_context(manager.queue_handle().clone());
    let (new_mod, site) = parse_lemmy_person(&context).await;
    let community = parse_lemmy_community(&context).await;
    let community_id = community.id;

    let old_mod = PersonForm {
      name: "holly".into(),
      ..PersonForm::default()
    };
    let old_mod = Person::create(&context.pool().get().unwrap(), &old_mod).unwrap();
    let community_moderator_form = CommunityModeratorForm {
      community_id: community.id,
      person_id: old_mod.id,
    };

    CommunityModerator::join(&context.pool().get().unwrap(), &community_moderator_form).unwrap();

    assert_eq!(site.actor_id.to_string(), "https://enterprise.lemmy.ml/");

    let json: GroupModerators =
      file_to_json_object("assets/lemmy/collections/group_moderators.json").unwrap();
    let url = Url::parse("https://enterprise.lemmy.ml/c/tenforward").unwrap();
    let mut request_counter = 0;
    let community_context = CommunityContext {
      0: community,
      1: context,
    };
    ApubCommunityModerators::verify(&json, &url, &community_context, &mut request_counter)
      .await
      .unwrap();
    ApubCommunityModerators::from_apub(json, &community_context, &mut request_counter)
      .await
      .unwrap();
    assert_eq!(request_counter, 0);

    let current_moderators = blocking(community_context.1.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await
    .unwrap()
    .unwrap();

    assert_eq!(current_moderators.len(), 1);
    assert_eq!(current_moderators[0].moderator.id, new_mod.id);

    Person::delete(&*community_context.1.pool().get().unwrap(), old_mod.id).unwrap();
    Person::delete(&*community_context.1.pool().get().unwrap(), new_mod.id).unwrap();
    Community::delete(
      &*community_context.1.pool().get().unwrap(),
      community_context.0.id,
    )
    .unwrap();
    Site::delete(&*community_context.1.pool().get().unwrap(), site.id).unwrap();
  }
}
