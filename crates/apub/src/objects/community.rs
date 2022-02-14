use crate::{
  check_is_apub_id_valid,
  collections::{community_moderators::ApubCommunityModerators, CommunityContext},
  generate_moderators_url,
  generate_outbox_url,
  objects::instance::fetch_instance_actor_for_object,
  protocol::{
    objects::{group::Group, tombstone::Tombstone, Endpoints},
    ImageObject,
    Source,
  },
};
use activitystreams_kinds::actor::GroupType;
use chrono::NaiveDateTime;
use itertools::Itertools;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  object_id::ObjectId,
  traits::{ActorType, ApubObject},
};
use lemmy_db_schema::{source::community::Community, traits::ApubActor};
use lemmy_db_views_actor::community_follower_view::CommunityFollowerView;
use lemmy_utils::{
  utils::{convert_datetime, markdown_to_html},
  LemmyError,
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
    ApubCommunity { 0: c }
  }
}

#[async_trait::async_trait(?Send)]
impl ApubObject for ApubCommunity {
  type DataType = LemmyContext;
  type ApubType = Group;
  type TombstoneType = Tombstone;

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
        Community::read_from_apub_id(conn, object_id)
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
      name: self.title.clone(),
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
      public_key: self.get_public_key()?,
      published: Some(convert_datetime(self.published)),
      updated: self.updated.map(convert_datetime),
    };
    Ok(group)
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    Ok(Tombstone::new(self.actor_id()))
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
      .dereference(&outbox_data, context.client(), request_counter)
      .await
      .map_err(|e| debug!("{}", e))
      .ok();

    if let Some(moderators) = &group.moderators {
      moderators
        .dereference(&outbox_data, context.client(), request_counter)
        .await
        .map_err(|e| debug!("{}", e))
        .ok();
    }

    fetch_instance_actor_for_object(community.actor_id(), context, request_counter).await;

    Ok(community)
  }
}

impl ActorType for ApubCommunity {
  fn actor_id(&self) -> Url {
    self.actor_id.to_owned().into()
  }
  fn public_key(&self) -> String {
    self.public_key.to_owned()
  }
  fn private_key(&self) -> Option<String> {
    self.private_key.to_owned()
  }

  fn inbox_url(&self) -> Url {
    self.inbox_url.clone().into()
  }

  fn shared_inbox_url(&self) -> Option<Url> {
    self.shared_inbox_url.clone().map(|s| s.into())
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
      .filter(|inbox| check_is_apub_id_valid(inbox, false, &context.settings()).is_ok())
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
  use lemmy_apub_lib::activity_queue::create_activity_queue;
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
    let client = reqwest::Client::new().into();
    let manager = create_activity_queue(client);
    let context = init_context(manager.queue_handle().clone());
    let site = parse_lemmy_instance(&context).await;
    let community = parse_lemmy_community(&context).await;

    assert_eq!(community.title, "Ten Forward");
    assert!(!community.local);
    assert_eq!(community.description.as_ref().unwrap().len(), 132);

    Community::delete(&*context.pool().get().unwrap(), community.id).unwrap();
    Site::delete(&*context.pool().get().unwrap(), site.id).unwrap();
  }
}
