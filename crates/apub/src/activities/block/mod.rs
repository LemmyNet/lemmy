use crate::{
  objects::{community::ApubCommunity, instance::ApubSite, person::ApubPerson},
  protocol::{
    activities::block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
    objects::{group::Group, instance::Instance},
  },
  SendActivity,
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  traits::{Actor, Object},
};
use chrono::NaiveDateTime;
use lemmy_api_common::{
  community::{BanFromCommunity, BanFromCommunityResponse},
  context::LemmyContext,
  person::{BanPerson, BanPersonResponse},
  utils::local_user_view_from_jwt,
};
use lemmy_db_schema::{
  source::{community::Community, person::Person, site::Site},
  traits::Crud,
  utils::DbPool,
};
use lemmy_db_views::structs::SiteView;
use lemmy_utils::{error::LemmyError, utils::time::naive_from_unix};
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
  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    Some(match self {
      SiteOrCommunity::Site(i) => i.last_refreshed_at,
      SiteOrCommunity::Community(c) => c.last_refreshed_at,
    })
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_id(
    object_id: Url,
    data: &Data<Self::DataType>,
  ) -> Result<Option<Self>, LemmyError>
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

  async fn delete(self, _data: &Data<Self::DataType>) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, LemmyError> {
    unimplemented!()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    apub: &Self::Kind,
    expected_domain: &Url,
    data: &Data<Self::DataType>,
  ) -> Result<(), LemmyError> {
    match apub {
      InstanceOrGroup::Instance(i) => ApubSite::verify(i, expected_domain, data).await,
      InstanceOrGroup::Group(g) => ApubCommunity::verify(g, expected_domain, data).await,
    }
  }

  #[tracing::instrument(skip_all)]
  async fn from_json(apub: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, LemmyError>
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

async fn generate_cc(
  target: &SiteOrCommunity,
  pool: &mut DbPool<'_>,
) -> Result<Vec<Url>, LemmyError> {
  Ok(match target {
    SiteOrCommunity::Site(_) => Site::read_remote_sites(pool)
      .await?
      .into_iter()
      .map(|s| s.actor_id.into())
      .collect(),
    SiteOrCommunity::Community(c) => vec![c.id()],
  })
}

#[async_trait::async_trait]
impl SendActivity for BanPerson {
  type Response = BanPersonResponse;

  async fn send_activity(
    request: &Self,
    _response: &Self::Response,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let local_user_view = local_user_view_from_jwt(&request.auth, context).await?;
    let person = Person::read(&mut context.pool(), request.person_id).await?;
    let site = SiteOrCommunity::Site(SiteView::read_local(&mut context.pool()).await?.site.into());
    let expires = request.expires.map(naive_from_unix);

    // if the action affects a local user, federate to other instances
    if person.local {
      if request.ban {
        BlockUser::send(
          &site,
          &person.into(),
          &local_user_view.person.into(),
          request.remove_data.unwrap_or(false),
          request.reason.clone(),
          expires,
          context,
        )
        .await
      } else {
        UndoBlockUser::send(
          &site,
          &person.into(),
          &local_user_view.person.into(),
          request.reason.clone(),
          context,
        )
        .await
      }
    } else {
      Ok(())
    }
  }
}

#[async_trait::async_trait]
impl SendActivity for BanFromCommunity {
  type Response = BanFromCommunityResponse;

  async fn send_activity(
    request: &Self,
    _response: &Self::Response,
    context: &Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let local_user_view = local_user_view_from_jwt(&request.auth, context).await?;
    let community: ApubCommunity = Community::read(&mut context.pool(), request.community_id)
      .await?
      .into();
    let banned_person: ApubPerson = Person::read(&mut context.pool(), request.person_id)
      .await?
      .into();
    let expires = request.expires.map(naive_from_unix);

    if request.ban {
      BlockUser::send(
        &SiteOrCommunity::Community(community),
        &banned_person,
        &local_user_view.person.clone().into(),
        request.remove_data.unwrap_or(false),
        request.reason.clone(),
        expires,
        context,
      )
      .await
    } else {
      UndoBlockUser::send(
        &SiteOrCommunity::Community(community),
        &banned_person,
        &local_user_view.person.clone().into(),
        request.reason.clone(),
        context,
      )
      .await
    }
  }
}
