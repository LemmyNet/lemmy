use crate::{
  check_apub_id_valid_with_strictness,
  collections::{community_moderators::ApubCommunityModerators, CommunityContext},
  generate_moderators_url,
  generate_outbox_url,
  local_instance,
  objects::instance::fetch_instance_actor_for_object,
  protocol::{
    objects::{group::Group, Endpoints},
    ImageObject,
    Source,
  },
  ActorType,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  traits::{Actor, ApubObject},
};
use activitystreams_kinds::actor::GroupType;
use chrono::NaiveDateTime;
use itertools::Itertools;
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{source::community::Community, traits::ApubActor};
use lemmy_db_views_actor::structs::CommunityFollowerView;
use lemmy_utils::{
  error::LemmyError,
  utils::{convert_datetime, markdown_to_html},
};
use lemmy_websocket::LemmyContext;
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

#[async_trait::async_trait(?Send)]
impl ApubObject for ApubCommunity {
  type DataType = LemmyContext;
  type ApubType = Group;
  type DbType = Community;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    Some(self.last_refreshed_at)
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_apub_id(
    object_id: Url,
    context: &LemmyContext,
  ) -> Result<Option<Self>, LemmyError> {
    Ok(
      blocking(context.pool(), move |conn| {
        Community::read_from_apub_id(conn, &object_id.into())
      })
      .await??
      .map(Into::into),
    )
  }

  #[tracing::instrument(skip_all)]
  async fn delete(self, context: &LemmyContext) -> Result<(), LemmyError> {
    blocking(context.pool(), move |conn| {
      Community::update_deleted(conn, self.id, true)
    })
    .await??;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn into_apub(self, _context: &LemmyContext) -> Result<Group, LemmyError> {
    let group = Group {
      kind: GroupType::Group,
      id: ObjectId::new(self.actor_id()),
      preferred_username: self.name.clone(),
      name: Some(self.title.clone()),
      summary: self.description.as_ref().map(|b| markdown_to_html(b)),
      source: self.description.clone().map(Source::new),
      icon: self.icon.clone().map(ImageObject::new),
      image: self.banner.clone().map(ImageObject::new),
      sensitive: Some(self.nsfw),
      moderators: Some(ObjectId::<ApubCommunityModerators>::new(
        generate_moderators_url(&self.actor_id)?,
      )),
      inbox: self.inbox_url.clone().into(),
      outbox: ObjectId::new(generate_outbox_url(&self.actor_id)?),
      followers: self.followers_url.clone().into(),
      endpoints: self.shared_inbox_url.clone().map(|s| Endpoints {
        shared_inbox: s.into(),
      }),
      public_key: self.get_public_key(),
      published: Some(convert_datetime(self.published)),
      updated: self.updated.map(convert_datetime),
      posting_restricted_to_mods: Some(self.posting_restricted_to_mods),
    };
    Ok(group)
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    group: &Group,
    expected_domain: &Url,
    context: &LemmyContext,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    group.verify(expected_domain, context).await
  }

  /// Converts a `Group` to `Community`, inserts it into the database and updates moderators.
  #[tracing::instrument(skip_all)]
  async fn from_apub(
    group: Group,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let form = Group::into_form(group.clone());

    // Fetching mods and outbox is not necessary for Lemmy to work, so ignore errors. Besides,
    // we need to ignore these errors so that tests can work entirely offline.
    let community: ApubCommunity =
      blocking(context.pool(), move |conn| Community::upsert(conn, &form))
        .await??
        .into();
    let outbox_data = CommunityContext(community.clone(), context.clone());

    group
      .outbox
      .dereference(&outbox_data, local_instance(context), request_counter)
      .await
      .map_err(|e| debug!("{}", e))
      .ok();

    if let Some(moderators) = &group.moderators {
      moderators
        .dereference(&outbox_data, local_instance(context), request_counter)
        .await
        .map_err(|e| debug!("{}", e))
        .ok();
    }

    fetch_instance_actor_for_object(community.actor_id(), context, request_counter).await;

    Ok(community)
  }
}

impl Actor for ApubCommunity {
  fn public_key(&self) -> &str {
    &self.public_key
  }

  fn inbox(&self) -> Url {
    self.inbox_url.clone().into()
  }

  fn shared_inbox(&self) -> Option<Url> {
    self.shared_inbox_url.clone().map(|s| s.into())
  }
}

impl ActorType for ApubCommunity {
  fn actor_id(&self) -> Url {
    self.actor_id.to_owned().into()
  }
  fn private_key(&self) -> Option<String> {
    self.private_key.to_owned()
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

    let follows = blocking(context.pool(), move |conn| {
      CommunityFollowerView::for_community(conn, id)
    })
    .await??;
    let inboxes: Vec<Url> = follows
      .into_iter()
      .filter(|f| !f.follower.local)
      .map(|f| {
        f.follower
          .shared_inbox_url
          .unwrap_or(f.follower.inbox_url)
          .into()
      })
      .unique()
      .filter(|inbox: &Url| inbox.host_str() != Some(&context.settings().hostname))
      // Don't send to blocked instances
      .filter(|inbox| check_apub_id_valid_with_strictness(inbox, false, context.settings()).is_ok())
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
  use lemmy_db_schema::{source::site::Site, traits::Crud};
  use serial_test::serial;

  pub(crate) async fn parse_lemmy_community(context: &LemmyContext) -> ApubCommunity {
    let mut json: Group = file_to_json_object("assets/lemmy/objects/group.json").unwrap();
    // change these links so they dont fetch over the network
    json.moderators = None;
    json.outbox =
      ObjectId::new(Url::parse("https://enterprise.lemmy.ml/c/tenforward/not_outbox").unwrap());

    let url = Url::parse("https://enterprise.lemmy.ml/c/tenforward").unwrap();
    let mut request_counter = 0;
    ApubCommunity::verify(&json, &url, context, &mut request_counter)
      .await
      .unwrap();
    let community = ApubCommunity::from_apub(json, context, &mut request_counter)
      .await
      .unwrap();
    // this makes one requests to the (intentionally broken) outbox collection
    assert_eq!(request_counter, 1);
    community
  }

  #[actix_rt::test]
  #[serial]
  async fn test_parse_lemmy_community() {
    let context = init_context();
    let conn = &mut context.pool().get().unwrap();
    let site = parse_lemmy_instance(&context).await;
    let community = parse_lemmy_community(&context).await;

    assert_eq!(community.title, "Ten Forward");
    assert!(!community.local);
    assert_eq!(community.description.as_ref().unwrap().len(), 132);

    Community::delete(conn, community.id).unwrap();
    Site::delete(conn, site.id).unwrap();
  }
}
