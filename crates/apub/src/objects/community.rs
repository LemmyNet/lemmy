use crate::{
  activities::GetActorType,
  check_apub_id_valid,
  local_site_data_cached,
  objects::instance::fetch_instance_actor_for_object,
  protocol::{
    objects::{group::Group, Endpoints, LanguageTag},
    ImageObject,
    Source,
  },
};
use activitypub_federation::{
  config::Data,
  kinds::actor::GroupType,
  traits::{Actor, Object},
};
use chrono::{DateTime, Utc};
use lemmy_api_common::{
  context::LemmyContext,
  utils::{generate_featured_url, generate_moderators_url, generate_outbox_url},
};
use lemmy_db_schema::{
  source::{
    activity::ActorType,
    actor_language::CommunityLanguage,
    community::{Community, CommunityUpdateForm},
  },
  traits::{ApubActor, Crud},
};
use lemmy_db_views_actor::structs::CommunityFollowerView;
use lemmy_utils::{error::LemmyError, utils::markdown::markdown_to_html};
use std::ops::Deref;
use tracing::debug;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubCommunity(Community);

impl Deref for ApubCommunity {
  type Target = Community;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Community> for ApubCommunity {
  fn from(c: Community) -> Self {
    ApubCommunity(c)
  }
}

#[async_trait::async_trait]
impl Object for ApubCommunity {
  type DataType = LemmyContext;
  type Kind = Group;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    Some(self.last_refreshed_at)
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_id(
    object_id: Url,
    context: &Data<Self::DataType>,
  ) -> Result<Option<Self>, LemmyError> {
    Ok(
      Community::read_from_apub_id(&mut context.pool(), &object_id.into())
        .await?
        .map(Into::into),
    )
  }

  #[tracing::instrument(skip_all)]
  async fn delete(self, context: &Data<Self::DataType>) -> Result<(), LemmyError> {
    let form = CommunityUpdateForm {
      deleted: Some(true),
      ..Default::default()
    };
    Community::update(&mut context.pool(), self.id, &form).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn into_json(self, data: &Data<Self::DataType>) -> Result<Group, LemmyError> {
    let community_id = self.id;
    let langs = CommunityLanguage::read(&mut data.pool(), community_id).await?;
    let language = LanguageTag::new_multiple(langs, &mut data.pool()).await?;

    let group = Group {
      kind: GroupType::Group,
      id: self.id().into(),
      preferred_username: self.name.clone(),
      name: Some(self.title.clone()),
      summary: self.description.as_ref().map(|b| markdown_to_html(b)),
      source: self.description.clone().map(Source::new),
      icon: self.icon.clone().map(ImageObject::new),
      image: self.banner.clone().map(ImageObject::new),
      sensitive: Some(self.nsfw),
      featured: Some(generate_featured_url(&self.actor_id)?.into()),
      inbox: self.inbox_url.clone().into(),
      outbox: generate_outbox_url(&self.actor_id)?.into(),
      followers: self.followers_url.clone().into(),
      endpoints: self.shared_inbox_url.clone().map(|s| Endpoints {
        shared_inbox: s.into(),
      }),
      public_key: self.public_key(),
      language,
      published: Some(self.published),
      updated: self.updated,
      posting_restricted_to_mods: Some(self.posting_restricted_to_mods),
      attributed_to: Some(generate_moderators_url(&self.actor_id)?.into()),
    };
    Ok(group)
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    group: &Group,
    expected_domain: &Url,
    context: &Data<Self::DataType>,
  ) -> Result<(), LemmyError> {
    group.verify(expected_domain, context).await
  }

  /// Converts a `Group` to `Community`, inserts it into the database and updates moderators.
  #[tracing::instrument(skip_all)]
  async fn from_json(
    group: Group,
    context: &Data<Self::DataType>,
  ) -> Result<ApubCommunity, LemmyError> {
    let instance_id = fetch_instance_actor_for_object(&group.id, context).await?;

    let form = Group::into_insert_form(group.clone(), instance_id);
    let languages =
      LanguageTag::to_language_id_multiple(group.language, &mut context.pool()).await?;

    let community = Community::create(&mut context.pool(), &form).await?;
    CommunityLanguage::update(&mut context.pool(), languages, community.id).await?;

    let community: ApubCommunity = community.into();

    // Fetching mods and outbox is not necessary for Lemmy to work, so ignore errors. Besides,
    // we need to ignore these errors so that tests can work entirely offline.
    let fetch_outbox = group.outbox.dereference(&community, context);
    let fetch_followers = group.followers.dereference(&community, context);

    if let Some(moderators) = group.attributed_to {
      let fetch_moderators = moderators.dereference(&community, context);
      // Fetch mods, outbox and followers in parallel
      let res = tokio::join!(fetch_outbox, fetch_moderators, fetch_followers);
      res.0.map_err(|e| debug!("{}", e)).ok();
      res.1.map_err(|e| debug!("{}", e)).ok();
      res.2.map_err(|e| debug!("{}", e)).ok();
    } else {
      let res = tokio::join!(fetch_outbox, fetch_followers);
      res.0.map_err(|e| debug!("{}", e)).ok();
      res.1.map_err(|e| debug!("{}", e)).ok();
    }

    Ok(community)
  }
}

impl Actor for ApubCommunity {
  fn id(&self) -> Url {
    self.actor_id.inner().clone()
  }

  fn public_key_pem(&self) -> &str {
    &self.public_key
  }

  fn private_key_pem(&self) -> Option<String> {
    self.private_key.clone()
  }

  fn inbox(&self) -> Url {
    self.inbox_url.clone().into()
  }

  fn shared_inbox(&self) -> Option<Url> {
    self.shared_inbox_url.clone().map(Into::into)
  }
}

impl GetActorType for ApubCommunity {
  fn actor_type(&self) -> ActorType {
    ActorType::Community
  }
}

impl ApubCommunity {
  /// For a given community, returns the inboxes of all followers.
  #[tracing::instrument(skip_all)]
  pub(crate) async fn get_follower_inboxes(
    &self,
    context: &LemmyContext,
  ) -> Result<Vec<Url>, LemmyError> {
    let id = self.id;

    let local_site_data = local_site_data_cached(&mut context.pool()).await?;
    let follows =
      CommunityFollowerView::get_community_follower_inboxes(&mut context.pool(), id).await?;
    let inboxes: Vec<Url> = follows
      .into_iter()
      .map(Into::into)
      .filter(|inbox: &Url| inbox.host_str() != Some(&context.settings().hostname))
      // Don't send to blocked instances
      .filter(|inbox| check_apub_id_valid(inbox, &local_site_data).is_ok())
      .collect();

    Ok(inboxes)
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::{
    objects::{instance::tests::parse_lemmy_instance, tests::init_context},
    protocol::tests::file_to_json_object,
  };
  use activitypub_federation::fetch::collection_id::CollectionId;
  use lemmy_db_schema::{source::site::Site, traits::Crud};
  use lemmy_utils::error::LemmyResult;
  use serial_test::serial;

  pub(crate) async fn parse_lemmy_community(
    context: &Data<LemmyContext>,
  ) -> LemmyResult<ApubCommunity> {
    // use separate counter so this doesnt affect tests
    let context2 = context.reset_request_count();
    let mut json: Group = file_to_json_object("assets/lemmy/objects/group.json")?;
    // change these links so they dont fetch over the network
    json.attributed_to = None;
    json.outbox = CollectionId::parse("https://enterprise.lemmy.ml/c/tenforward/not_outbox")?;
    json.followers = CollectionId::parse("https://enterprise.lemmy.ml/c/tenforward/not_followers")?;

    let url = Url::parse("https://enterprise.lemmy.ml/c/tenforward")?;
    ApubCommunity::verify(&json, &url, &context2).await?;
    let community = ApubCommunity::from_json(json, &context2).await?;
    // this makes requests to the (intentionally broken) outbox and followers collections
    assert_eq!(context2.request_count(), 2);
    Ok(community)
  }

  #[tokio::test]
  #[serial]
  async fn test_parse_lemmy_community() -> LemmyResult<()> {
    let context = init_context().await?;
    let site = parse_lemmy_instance(&context).await?;
    let community = parse_lemmy_community(&context).await?;

    assert_eq!(community.title, "Ten Forward");
    assert!(!community.local);
    assert_eq!(
      community.description.as_ref().map(std::string::String::len),
      Some(132)
    );

    Community::delete(&mut context.pool(), community.id).await?;
    Site::delete(&mut context.pool(), site.id).await?;
    Ok(())
  }
}
