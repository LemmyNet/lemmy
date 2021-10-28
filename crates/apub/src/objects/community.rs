use crate::{
  check_is_apub_id_valid,
  collections::{
    community_moderators::ApubCommunityModerators,
    community_outbox::ApubCommunityOutbox,
    CommunityContext,
  },
  context::lemmy_context,
  fetcher::object_id::ObjectId,
  generate_moderators_url,
  generate_outbox_url,
  objects::{get_summary_from_string_or_source, tombstone::Tombstone, ImageObject, Source},
};
use activitystreams::{
  actor::{kind::GroupType, Endpoints},
  base::AnyBase,
  chrono::NaiveDateTime,
  object::kind::ImageType,
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use chrono::{DateTime, FixedOffset};
use itertools::Itertools;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  signatures::PublicKey,
  traits::{ActorType, ApubObject},
  values::MediaTypeMarkdown,
  verify::verify_domains_match,
};
use lemmy_db_schema::{
  naive_now,
  source::community::{Community, CommunityForm},
  DbPool,
};
use lemmy_db_views_actor::community_follower_view::CommunityFollowerView;
use lemmy_utils::{
  settings::structs::Settings,
  utils::{check_slurs, check_slurs_opt, convert_datetime, markdown_to_html},
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::ops::Deref;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(rename = "type")]
  kind: GroupType,
  pub(crate) id: Url,
  /// username, set at account creation and can never be changed
  preferred_username: String,
  /// title (can be changed at any time)
  name: String,
  summary: Option<String>,
  source: Option<Source>,
  icon: Option<ImageObject>,
  /// banner
  image: Option<ImageObject>,
  // lemmy extension
  sensitive: Option<bool>,
  // lemmy extension
  pub(crate) moderators: Option<ObjectId<ApubCommunityModerators>>,
  inbox: Url,
  pub(crate) outbox: ObjectId<ApubCommunityOutbox>,
  followers: Url,
  endpoints: Endpoints<Url>,
  public_key: PublicKey,
  published: Option<DateTime<FixedOffset>>,
  updated: Option<DateTime<FixedOffset>>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

impl Group {
  pub(crate) async fn from_apub_to_form(
    group: &Group,
    expected_domain: &Url,
    settings: &Settings,
  ) -> Result<CommunityForm, LemmyError> {
    verify_domains_match(expected_domain, &group.id)?;
    let name = group.preferred_username.clone();
    let title = group.name.clone();
    let description = get_summary_from_string_or_source(&group.summary, &group.source);
    let shared_inbox = group.endpoints.shared_inbox.clone().map(|s| s.into());

    let slur_regex = &settings.slur_regex();
    check_slurs(&name, slur_regex)?;
    check_slurs(&title, slur_regex)?;
    check_slurs_opt(&description, slur_regex)?;

    Ok(CommunityForm {
      name,
      title,
      description,
      removed: None,
      published: group.published.map(|u| u.naive_local()),
      updated: group.updated.map(|u| u.naive_local()),
      deleted: None,
      nsfw: Some(group.sensitive.unwrap_or(false)),
      actor_id: Some(group.id.clone().into()),
      local: Some(false),
      private_key: None,
      public_key: Some(group.public_key.public_key_pem.clone()),
      last_refreshed_at: Some(naive_now()),
      icon: Some(group.icon.clone().map(|i| i.url.into())),
      banner: Some(group.image.clone().map(|i| i.url.into())),
      followers_url: Some(group.followers.clone().into()),
      inbox_url: Some(group.inbox.clone().into()),
      shared_inbox_url: Some(shared_inbox),
    })
  }
}

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

  async fn delete(self, context: &LemmyContext) -> Result<(), LemmyError> {
    blocking(context.pool(), move |conn| {
      Community::update_deleted(conn, self.id, true)
    })
    .await??;
    Ok(())
  }

  async fn to_apub(&self, _context: &LemmyContext) -> Result<Group, LemmyError> {
    let source = self.description.clone().map(|bio| Source {
      content: bio,
      media_type: MediaTypeMarkdown::Markdown,
    });
    let icon = self.icon.clone().map(|url| ImageObject {
      kind: ImageType::Image,
      url: url.into(),
    });
    let image = self.banner.clone().map(|url| ImageObject {
      kind: ImageType::Image,
      url: url.into(),
    });

    let group = Group {
      context: lemmy_context(),
      kind: GroupType::Group,
      id: self.actor_id(),
      preferred_username: self.name.clone(),
      name: self.title.clone(),
      summary: self.description.as_ref().map(|b| markdown_to_html(b)),
      source,
      icon,
      image,
      sensitive: Some(self.nsfw),
      moderators: Some(ObjectId::<ApubCommunityModerators>::new(
        generate_moderators_url(&self.actor_id)?.into_inner(),
      )),
      inbox: self.inbox_url.clone().into(),
      outbox: ObjectId::new(generate_outbox_url(&self.actor_id)?),
      followers: self.followers_url.clone().into(),
      endpoints: Endpoints {
        shared_inbox: self.shared_inbox_url.clone().map(|s| s.into()),
        ..Default::default()
      },
      public_key: self.get_public_key()?,
      published: Some(convert_datetime(self.published)),
      updated: self.updated.map(convert_datetime),
      unparsed: Default::default(),
    };
    Ok(group)
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    Ok(Tombstone::new(
      GroupType::Group,
      self.updated.unwrap_or(self.published),
    ))
  }

  /// Converts a `Group` to `Community`, inserts it into the database and updates moderators.
  async fn from_apub(
    group: &Group,
    context: &LemmyContext,
    expected_domain: &Url,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let form = Group::from_apub_to_form(group, expected_domain, &context.settings()).await?;

    // Fetching mods and outbox is not necessary for Lemmy to work, so ignore errors. Besides,
    // we need to ignore these errors so that tests can work entirely offline.
    let community: ApubCommunity =
      blocking(context.pool(), move |conn| Community::upsert(conn, &form))
        .await??
        .into();
    let outbox_data = CommunityContext(community.clone(), context.clone());

    group
      .outbox
      .dereference(&outbox_data, request_counter)
      .await
      .map_err(|e| debug!("{}", e))
      .ok();

    if let Some(moderators) = &group.moderators {
      moderators
        .dereference(&outbox_data, request_counter)
        .await
        .map_err(|e| debug!("{}", e))
        .ok();
    }

    Ok(community)
  }
}

impl ActorType for ApubCommunity {
  fn is_local(&self) -> bool {
    self.local
  }
  fn actor_id(&self) -> Url {
    self.actor_id.to_owned().into()
  }
  fn name(&self) -> String {
    self.name.clone()
  }
  fn public_key(&self) -> Option<String> {
    self.public_key.to_owned()
  }
  fn private_key(&self) -> Option<String> {
    self.private_key.to_owned()
  }

  fn inbox_url(&self) -> Url {
    self.inbox_url.clone().into()
  }

  fn shared_inbox_url(&self) -> Option<Url> {
    self.shared_inbox_url.clone().map(|s| s.into_inner())
  }
}

impl ApubCommunity {
  /// For a given community, returns the inboxes of all followers.
  pub(crate) async fn get_follower_inboxes(
    &self,
    pool: &DbPool,
    settings: &Settings,
  ) -> Result<Vec<Url>, LemmyError> {
    let id = self.id;

    let follows = blocking(pool, move |conn| {
      CommunityFollowerView::for_community(conn, id)
    })
    .await??;
    let inboxes = follows
      .into_iter()
      .filter(|f| !f.follower.local)
      .map(|f| f.follower.shared_inbox_url.unwrap_or(f.follower.inbox_url))
      .map(|i| i.into_inner())
      .unique()
      // Don't send to blocked instances
      .filter(|inbox| check_is_apub_id_valid(inbox, false, settings).is_ok())
      .collect();

    Ok(inboxes)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::objects::tests::{file_to_json_object, init_context};
  use assert_json_diff::assert_json_include;
  use lemmy_db_schema::traits::Crud;
  use serial_test::serial;

  #[actix_rt::test]
  #[serial]
  async fn test_parse_lemmy_community() {
    let context = init_context();
    let mut json: Group = file_to_json_object("assets/lemmy-community.json");
    let json_orig = json.clone();
    // change these links so they dont fetch over the network
    json.moderators = Some(ObjectId::new(
      Url::parse("https://enterprise.lemmy.ml/c/tenforward/not_moderators").unwrap(),
    ));
    json.outbox =
      ObjectId::new(Url::parse("https://enterprise.lemmy.ml/c/tenforward/not_outbox").unwrap());

    let url = Url::parse("https://enterprise.lemmy.ml/c/tenforward").unwrap();
    let mut request_counter = 0;
    let community = ApubCommunity::from_apub(&json, &context, &url, &mut request_counter)
      .await
      .unwrap();

    assert_eq!(community.actor_id.clone().into_inner(), url);
    assert_eq!(community.title, "Ten Forward");
    assert!(community.public_key.is_some());
    assert!(!community.local);
    assert_eq!(community.description.as_ref().unwrap().len(), 132);
    // this makes two requests to the (intentionally) broken outbox/moderators collections
    assert_eq!(request_counter, 2);

    let to_apub = community.to_apub(&context).await.unwrap();
    assert_json_include!(actual: json_orig, expected: to_apub);

    Community::delete(&*context.pool().get().unwrap(), community.id).unwrap();
  }
}
