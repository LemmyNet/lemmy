use crate::protocol::collections::group_moderators::GroupModerators;
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::collection::OrderedCollectionType,
  protocol::verification::verify_domains_match,
  traits::Collection,
};
use lemmy_api_common::{context::LemmyContext, utils::generate_moderators_url};
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, person::ApubPerson},
  utils::functions::handle_community_moderators,
};
use lemmy_db_views_community_moderator::CommunityModeratorView;
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

  async fn read_local(owner: &Self::Owner, data: &Data<Self::DataType>) -> LemmyResult<Self::Kind> {
    let moderators = CommunityModeratorView::for_community(&mut data.pool(), owner.id).await?;
    let ordered_items = moderators
      .into_iter()
      .map(|m| ObjectId::<ApubPerson>::from(m.moderator.ap_id))
      .collect();
    Ok(GroupModerators {
      r#type: OrderedCollectionType::OrderedCollection,
      id: generate_moderators_url(&owner.ap_id)?.into(),
      ordered_items,
    })
  }

  async fn verify(
    group_moderators: &GroupModerators,
    expected_domain: &Url,
    _data: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    verify_domains_match(&group_moderators.id, expected_domain)?;
    Ok(())
  }

  async fn from_json(
    apub: Self::Kind,
    owner: &Self::Owner,
    data: &Data<Self::DataType>,
  ) -> LemmyResult<Self> {
    handle_community_moderators(&apub.ordered_items, owner, data).await?;

    // This return value is unused, so just set an empty vec
    Ok(ApubCommunityModerators(()))
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use super::*;
  use lemmy_apub_objects::utils::test::{
    file_to_json_object,
    parse_lemmy_community,
    parse_lemmy_person,
  };
  use lemmy_db_schema::{
    source::{
      community::{Community, CommunityActions, CommunityModeratorForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      site::Site,
    },
    traits::{Crud, Joinable},
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

    let old_mod = PersonInsertForm::test_form(inserted_instance.id, "holly");

    let old_mod = Person::create(&mut context.pool(), &old_mod).await?;
    let community_moderator_form = CommunityModeratorForm::new(community.id, old_mod.id);

    CommunityActions::join(&mut context.pool(), &community_moderator_form).await?;

    assert_eq!(site.ap_id.to_string(), "https://enterprise.lemmy.ml/");

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
