use crate::{
  extensions::{context::lemmy_context, group_extensions::GroupExtension},
  fetcher::user::get_or_fetch_and_upsert_user,
  objects::{
    check_object_domain,
    create_tombstone,
    get_object_from_apub,
    get_source_markdown_value,
    set_content_and_source,
    FromApub,
    FromApubToForm,
    ToApub,
  },
  ActorType,
  GroupExt,
};
use activitystreams::{
  actor::{kind::GroupType, ApActor, Endpoints, Group},
  base::BaseExt,
  object::{ApObject, Image, Tombstone},
  prelude::*,
};
use activitystreams_ext::Ext2;
use anyhow::Context;
use lemmy_api_structs::blocking;
use lemmy_db_queries::DbPool;
use lemmy_db_schema::{
  naive_now,
  source::community::{Community, CommunityForm},
};
use lemmy_db_views_actor::community_moderator_view::CommunityModeratorView;
use lemmy_utils::{
  location_info,
  utils::{check_slurs, check_slurs_opt, convert_datetime},
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use url::Url;

#[async_trait::async_trait(?Send)]
impl ToApub for Community {
  type ApubType = GroupExt;

  async fn to_apub(&self, pool: &DbPool) -> Result<GroupExt, LemmyError> {
    // The attributed to, is an ordered vector with the creator actor_ids first,
    // then the rest of the moderators
    // TODO Technically the instance admins can mod the community, but lets
    // ignore that for now
    let id = self.id;
    let moderators = blocking(pool, move |conn| {
      CommunityModeratorView::for_community(&conn, id)
    })
    .await??;
    let moderators: Vec<Url> = moderators
      .into_iter()
      .map(|m| m.moderator.actor_id.into_inner())
      .collect();

    let mut group = ApObject::new(Group::new());
    group
      .set_many_contexts(lemmy_context()?)
      .set_id(self.actor_id.to_owned().into())
      .set_name(self.title.to_owned())
      .set_published(convert_datetime(self.published))
      .set_many_attributed_tos(moderators);

    if let Some(u) = self.updated.to_owned() {
      group.set_updated(convert_datetime(u));
    }
    if let Some(d) = self.description.to_owned() {
      set_content_and_source(&mut group, &d)?;
    }

    if let Some(icon_url) = &self.icon {
      let mut image = Image::new();
      image.set_url(Url::parse(icon_url)?);
      group.set_icon(image.into_any_base()?);
    }

    if let Some(banner_url) = &self.banner {
      let mut image = Image::new();
      image.set_url(Url::parse(banner_url)?);
      group.set_image(image.into_any_base()?);
    }

    let mut ap_actor = ApActor::new(self.inbox_url.clone().into(), group);
    ap_actor
      .set_preferred_username(self.name.to_owned())
      .set_outbox(self.get_outbox_url()?)
      .set_followers(self.followers_url.clone().into())
      .set_endpoints(Endpoints {
        shared_inbox: Some(self.get_shared_inbox_or_inbox_url()),
        ..Default::default()
      });

    Ok(Ext2::new(
      ap_actor,
      GroupExtension::new(self.nsfw)?,
      self.get_public_key_ext()?,
    ))
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    create_tombstone(
      self.deleted,
      self.actor_id.to_owned().into(),
      self.updated,
      GroupType::Group,
    )
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for Community {
  type ApubType = GroupExt;

  /// Converts a `Group` to `Community`.
  async fn from_apub(
    group: &GroupExt,
    context: &LemmyContext,
    expected_domain: Url,
    request_counter: &mut i32,
  ) -> Result<Community, LemmyError> {
    get_object_from_apub(group, context, expected_domain, request_counter).await
  }
}

#[async_trait::async_trait(?Send)]
impl FromApubToForm<GroupExt> for CommunityForm {
  async fn from_apub(
    group: &GroupExt,
    context: &LemmyContext,
    expected_domain: Url,
    request_counter: &mut i32,
  ) -> Result<Self, LemmyError> {
    let creator_and_moderator_uris = group.inner.attributed_to().context(location_info!())?;
    let creator_uri = creator_and_moderator_uris
      .as_many()
      .context(location_info!())?
      .iter()
      .next()
      .context(location_info!())?
      .as_xsd_any_uri()
      .context(location_info!())?;

    let creator = get_or_fetch_and_upsert_user(creator_uri, context, request_counter).await?;
    let name = group
      .inner
      .preferred_username()
      .context(location_info!())?
      .to_string();
    let title = group
      .inner
      .name()
      .context(location_info!())?
      .as_one()
      .context(location_info!())?
      .as_xsd_string()
      .context(location_info!())?
      .to_string();

    let description = get_source_markdown_value(group)?;

    check_slurs(&name)?;
    check_slurs(&title)?;
    check_slurs_opt(&description)?;

    let icon = match group.icon() {
      Some(any_image) => Some(
        Image::from_any_base(any_image.as_one().context(location_info!())?.clone())
          .context(location_info!())?
          .context(location_info!())?
          .url()
          .context(location_info!())?
          .as_single_xsd_any_uri()
          .map(|u| u.to_string()),
      ),
      None => None,
    };
    let banner = match group.image() {
      Some(any_image) => Some(
        Image::from_any_base(any_image.as_one().context(location_info!())?.clone())
          .context(location_info!())?
          .context(location_info!())?
          .url()
          .context(location_info!())?
          .as_single_xsd_any_uri()
          .map(|u| u.to_string()),
      ),
      None => None,
    };
    let shared_inbox = group
      .inner
      .endpoints()?
      .map(|e| e.shared_inbox)
      .flatten()
      .map(|s| s.to_owned().into());

    Ok(CommunityForm {
      name,
      title,
      description,
      creator_id: creator.id,
      removed: None,
      published: group.inner.published().map(|u| u.to_owned().naive_local()),
      updated: group.inner.updated().map(|u| u.to_owned().naive_local()),
      deleted: None,
      nsfw: group.ext_one.sensitive.unwrap_or(false),
      actor_id: Some(check_object_domain(group, expected_domain)?),
      local: false,
      private_key: None,
      public_key: Some(group.ext_two.to_owned().public_key.public_key_pem),
      last_refreshed_at: Some(naive_now()),
      icon,
      banner,
      followers_url: Some(
        group
          .inner
          .followers()?
          .context(location_info!())?
          .to_owned()
          .into(),
      ),
      inbox_url: Some(group.inner.inbox()?.to_owned().into()),
      shared_inbox_url: Some(shared_inbox),
    })
  }
}
