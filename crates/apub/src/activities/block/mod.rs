use crate::{
  objects::{community::ApubCommunity, instance::ApubSite},
  protocol::{
    activities::block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
    objects::{group::Group, instance::Instance},
  },
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  traits::{Actor, Object},
};
use chrono::{DateTime, Utc};
use lemmy_api_common::{
  community::BanFromCommunity,
  context::LemmyContext,
  utils::check_expire_time,
};
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::{community::Community, person::Person, site::Site},
  traits::Crud,
  utils::DbPool,
};
use lemmy_db_views::structs::SiteView;
use lemmy_utils::{
  error::{LemmyError, LemmyResult},
  LemmyErrorType,
};
use serde::Deserialize;
use url::Url;

pub mod block_user;
pub mod undo_block_user;

#[derive(Clone, Debug)]
pub enum SiteOrCommunity {
  Site(ApubSite),
  Community(ApubCommunity),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum InstanceOrGroup {
  Instance(Instance),
  Group(Group),
}

#[async_trait::async_trait]
impl Object for SiteOrCommunity {
  type DataType = LemmyContext;
  type Kind = InstanceOrGroup;
  type Error = LemmyError;

  #[tracing::instrument(skip_all)]
  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    Some(match self {
      SiteOrCommunity::Site(i) => i.last_refreshed_at,
      SiteOrCommunity::Community(c) => c.last_refreshed_at,
    })
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_id(object_id: Url, data: &Data<Self::DataType>) -> LemmyResult<Option<Self>>
  where
    Self: Sized,
  {
    let site = ApubSite::read_from_id(object_id.clone(), data).await?;
    Ok(match site {
      Some(o) => Some(SiteOrCommunity::Site(o)),
      None => ApubCommunity::read_from_id(object_id, data)
        .await?
        .map(SiteOrCommunity::Community),
    })
  }

  async fn delete(self, _data: &Data<Self::DataType>) -> LemmyResult<()> {
    unimplemented!()
  }

  async fn into_json(self, _data: &Data<Self::DataType>) -> LemmyResult<Self::Kind> {
    unimplemented!()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    apub: &Self::Kind,
    expected_domain: &Url,
    data: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    match apub {
      InstanceOrGroup::Instance(i) => ApubSite::verify(i, expected_domain, data).await,
      InstanceOrGroup::Group(g) => ApubCommunity::verify(g, expected_domain, data).await,
    }
  }

  #[tracing::instrument(skip_all)]
  async fn from_json(apub: Self::Kind, data: &Data<Self::DataType>) -> LemmyResult<Self>
  where
    Self: Sized,
  {
    Ok(match apub {
      InstanceOrGroup::Instance(p) => SiteOrCommunity::Site(ApubSite::from_json(p, data).await?),
      InstanceOrGroup::Group(n) => {
        SiteOrCommunity::Community(ApubCommunity::from_json(n, data).await?)
      }
    })
  }
}

impl SiteOrCommunity {
  fn id(&self) -> ObjectId<SiteOrCommunity> {
    match self {
      SiteOrCommunity::Site(s) => ObjectId::from(s.actor_id.clone()),
      SiteOrCommunity::Community(c) => ObjectId::from(c.actor_id.clone()),
    }
  }
}

async fn generate_cc(target: &SiteOrCommunity, pool: &mut DbPool<'_>) -> LemmyResult<Vec<Url>> {
  Ok(match target {
    SiteOrCommunity::Site(_) => Site::read_remote_sites(pool)
      .await?
      .into_iter()
      .map(|s| s.actor_id.into())
      .collect(),
    SiteOrCommunity::Community(c) => vec![c.id()],
  })
}

pub(crate) async fn send_ban_from_site(
  moderator: Person,
  banned_user: Person,
  reason: Option<String>,
  remove_or_restore_data: Option<bool>,
  ban: bool,
  expires: Option<i64>,
  context: Data<LemmyContext>,
) -> LemmyResult<()> {
  let site = SiteOrCommunity::Site(
    SiteView::read_local(&mut context.pool())
      .await?
      .ok_or(LemmyErrorType::LocalSiteNotSetup)?
      .site
      .into(),
  );
  let expires = check_expire_time(expires)?;

  // if the action affects a local user, federate to other instances
  if banned_user.local {
    if ban {
      BlockUser::send(
        &site,
        &banned_user.into(),
        &moderator.into(),
        remove_or_restore_data.unwrap_or(false),
        reason.clone(),
        expires,
        &context,
      )
      .await
    } else {
      UndoBlockUser::send(
        &site,
        &banned_user.into(),
        &moderator.into(),
        remove_or_restore_data.unwrap_or(false),
        reason.clone(),
        &context,
      )
      .await
    }
  } else {
    Ok(())
  }
}

pub(crate) async fn send_ban_from_community(
  mod_: Person,
  community_id: CommunityId,
  banned_person: Person,
  data: BanFromCommunity,
  context: Data<LemmyContext>,
) -> LemmyResult<()> {
  let community: ApubCommunity = Community::read(&mut context.pool(), community_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindCommunity)?
    .into();
  let expires = check_expire_time(data.expires)?;

  if data.ban {
    BlockUser::send(
      &SiteOrCommunity::Community(community),
      &banned_person.into(),
      &mod_.into(),
      data.remove_or_restore_data.unwrap_or(false),
      data.reason.clone(),
      expires,
      &context,
    )
    .await
  } else {
    UndoBlockUser::send(
      &SiteOrCommunity::Community(community),
      &banned_person.into(),
      &mod_.into(),
      data.remove_or_restore_data.unwrap_or(false),
      data.reason.clone(),
      &context,
    )
    .await
  }
}
