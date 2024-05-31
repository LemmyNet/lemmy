use crate::{
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::collections::group_moderators::GroupModerators,
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::collection::OrderedCollectionType,
  protocol::verification::verify_domains_match,
  traits::Collection,
};
use lemmy_api_common::{context::LemmyContext, utils::generate_moderators_url};
use lemmy_db_schema::{
  source::community::{CommunityModerator, CommunityModeratorForm},
  traits::Joinable,
};
use lemmy_db_views_actor::structs::CommunityModeratorView;
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

#[derive(Clone, Debug)]
pub(crate) struct ApubCommunityModerators(());

#[async_trait::async_trait]
impl Collection for ApubCommunityModerators {
  type Owner = ApubCommunity;
  type DataType = LemmyContext;
  type Kind = GroupModerators;
  type Error = LemmyError;

  #[tracing::instrument(skip_all)]
  async fn read_local(owner: &Self::Owner, data: &Data<Self::DataType>) -> LemmyResult<Self::Kind> {
    let moderators = CommunityModeratorView::for_community(&mut data.pool(), owner.id).await?;
    let ordered_items = moderators
      .into_iter()
      .map(|m| ObjectId::<ApubPerson>::from(m.moderator.actor_id))
      .collect();
    Ok(GroupModerators {
      r#type: OrderedCollectionType::OrderedCollection,
      id: generate_moderators_url(&owner.actor_id)?.into(),
      ordered_items,
    })
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    group_moderators: &GroupModerators,
    expected_domain: &Url,
    _data: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    verify_domains_match(&group_moderators.id, expected_domain)?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn from_json(
    apub: Self::Kind,
    owner: &Self::Owner,
    data: &Data<Self::DataType>,
  ) -> LemmyResult<Self> {
    let community_id = owner.id;
    let current_moderators =
      CommunityModeratorView::for_community(&mut data.pool(), community_id).await?;
    // Remove old mods from database which arent in the moderators collection anymore
    for mod_user in &current_moderators {
      let mod_id = ObjectId::from(mod_user.moderator.actor_id.clone());
      if !apub.ordered_items.contains(&mod_id) {
        let community_moderator_form = CommunityModeratorForm {
          community_id: mod_user.community.id,
          person_id: mod_user.moderator.id,
        };
        let c = CommunityModerator::leave(&mut data.pool(), &community_moderator_form).await?;
        assert_eq!(c.deleted + c.updated, 1);
      }
    }
    let current_moderators =
      CommunityModeratorView::for_community(&mut data.pool(), community_id).await?;
    for mod_user in &current_moderators {
      let mod_id = ObjectId::from(mod_user.moderator.actor_id.clone());
      if !apub.ordered_items.contains(&mod_id) {
        panic!("pain");
      }
    }

    // Add new mods to database which have been added to moderators collection
    for mod_id in apub.ordered_items {
      // Ignore errors as mod accounts might be deleted or instances unavailable.
      let mod_user: Option<ApubPerson> = mod_id.dereference(data).await.ok();
      if let Some(mod_user) = mod_user {
        if !current_moderators
          .iter()
          .map(|c| c.moderator.actor_id.clone())
          .any(|x| x == mod_user.actor_id)
        {
          let community_moderator_form = CommunityModeratorForm {
            community_id: owner.id,
            person_id: mod_user.id,
          };
          CommunityModerator::join(&mut data.pool(), &community_moderator_form).await?;
        }
      }
    }

    // This return value is unused, so just set an empty vec
    Ok(ApubCommunityModerators(()))
  }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use super::*;
  use crate::{
    objects::{community::tests::parse_lemmy_community, person::tests::parse_lemmy_person},
    protocol::tests::file_to_json_object,
  };
  use lemmy_db_schema::{
    source::{
      community::Community,
      instance::Instance,
      person::{Person, PersonInsertForm},
      site::Site,
    },
    traits::Crud,
  };
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_parse_lemmy_community_moderators() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let (new_mod, site) = parse_lemmy_person(&context).await?;
    let community = parse_lemmy_community(&context).await?;
    let community_id = community.id;

    let inserted_instance =
      Instance::read_or_create(&mut context.pool(), "my_domain.tld".to_string()).await?;

    let old_mod = PersonInsertForm::builder()
      .name("holly".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let old_mod = Person::create(&mut context.pool(), &old_mod).await?;
    let community_moderator_form = CommunityModeratorForm {
      community_id: community.id,
      person_id: old_mod.id,
    };

    CommunityModerator::join(&mut context.pool(), &community_moderator_form).await?;

    assert_eq!(site.actor_id.to_string(), "https://enterprise.lemmy.ml/");

    let json: GroupModerators =
      file_to_json_object("assets/lemmy/collections/group_moderators.json")?;
    let url = Url::parse("https://enterprise.lemmy.ml/c/tenforward")?;
    ApubCommunityModerators::verify(&json, &url, &context).await?;
    ApubCommunityModerators::from_json(json, &community, &context).await?;
    assert_eq!(context.request_count(), 0);

    let current_moderators =
      CommunityModeratorView::for_community(&mut context.pool(), community_id).await?;

    assert_eq!(current_moderators.len(), 1);
    assert_eq!(current_moderators[0].moderator.id, new_mod.id);

    Person::delete(&mut context.pool(), old_mod.id).await?;
    Person::delete(&mut context.pool(), new_mod.id).await?;
    Community::delete(&mut context.pool(), community.id).await?;
    Site::delete(&mut context.pool(), site.id).await?;
    Instance::delete(&mut context.pool(), inserted_instance.id).await?;
    Ok(())
  }
}
