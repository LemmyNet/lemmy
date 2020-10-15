use crate::{
  extensions::group_extensions::GroupExtension,
  fetcher::get_or_fetch_and_upsert_user,
  objects::{check_object_domain, create_tombstone},
  ActorType,
  FromApub,
  GroupExt,
  ToApub,
};
use activitystreams::{
  actor::{kind::GroupType, ApActor, Endpoints, Group},
  base::BaseExt,
  object::{Image, Tombstone},
  prelude::*,
};
use activitystreams_ext::Ext2;
use anyhow::Context;
use lemmy_db::{
  community::{Community, CommunityForm},
  community_view::CommunityModeratorView,
  naive_now,
  DbPool,
};
use lemmy_structs::blocking;
use lemmy_utils::{
  location_info,
  utils::{check_slurs, check_slurs_opt, convert_datetime},
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use url::Url;

#[async_trait::async_trait(?Send)]
impl ToApub for Community {
  type Response = GroupExt;

  // Turn a Lemmy Community into an ActivityPub group that can be sent out over the network.
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
    let moderators: Vec<String> = moderators.into_iter().map(|m| m.user_actor_id).collect();

    let mut group = Group::new();
    group
      .set_context(activitystreams::context())
      .set_id(Url::parse(&self.actor_id)?)
      .set_name(self.name.to_owned())
      .set_published(convert_datetime(self.published))
      .set_many_attributed_tos(moderators);

    if let Some(u) = self.updated.to_owned() {
      group.set_updated(convert_datetime(u));
    }
    if let Some(d) = self.description.to_owned() {
      // TODO: this should be html, also add source field with raw markdown
      //       -> same for post.content and others
      group.set_content(d);
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

    let mut ap_actor = ApActor::new(self.get_inbox_url()?, group);
    ap_actor
      .set_preferred_username(self.title.to_owned())
      .set_outbox(self.get_outbox_url()?)
      .set_followers(self.get_followers_url()?)
      .set_endpoints(Endpoints {
        shared_inbox: Some(self.get_shared_inbox_url()?),
        ..Default::default()
      });

    let nsfw = self.nsfw;
    let category_id = self.category_id;
    let group_extension = blocking(pool, move |conn| {
      GroupExtension::new(conn, category_id, nsfw)
    })
    .await??;

    Ok(Ext2::new(
      ap_actor,
      group_extension,
      self.get_public_key_ext()?,
    ))
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    create_tombstone(self.deleted, &self.actor_id, self.updated, GroupType::Group)
  }
}
#[async_trait::async_trait(?Send)]
impl FromApub for CommunityForm {
  type ApubType = GroupExt;

  /// Parse an ActivityPub group received from another instance into a Lemmy community.
  async fn from_apub(
    group: &GroupExt,
    context: &LemmyContext,
    expected_domain: Option<Url>,
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

    let creator = get_or_fetch_and_upsert_user(creator_uri, context).await?;
    let name = group
      .inner
      .name()
      .context(location_info!())?
      .as_one()
      .context(location_info!())?
      .as_xsd_string()
      .context(location_info!())?
      .to_string();
    let title = group
      .inner
      .preferred_username()
      .context(location_info!())?
      .to_string();
    // TODO: should be parsed as html and tags like <script> removed (or use markdown source)
    //       -> same for post.content etc
    let description = group
      .inner
      .content()
      .map(|s| s.as_single_xsd_string())
      .flatten()
      .map(|s| s.to_string());
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

    Ok(CommunityForm {
      name,
      title,
      description,
      category_id: group.ext_one.category.identifier.parse::<i32>()?,
      creator_id: creator.id,
      removed: None,
      published: group.inner.published().map(|u| u.to_owned().naive_local()),
      updated: group.inner.updated().map(|u| u.to_owned().naive_local()),
      deleted: None,
      nsfw: group.ext_one.sensitive,
      actor_id: Some(check_object_domain(group, expected_domain)?),
      local: false,
      private_key: None,
      public_key: Some(group.ext_two.to_owned().public_key.public_key_pem),
      last_refreshed_at: Some(naive_now()),
      icon,
      banner,
    })
  }
}
