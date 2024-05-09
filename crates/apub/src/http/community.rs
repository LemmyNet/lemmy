use crate::{
  activity_lists::GroupInboxActivities,
  collections::{
    community_featured::ApubCommunityFeatured,
    community_follower::ApubCommunityFollower,
    community_moderators::ApubCommunityModerators,
    community_outbox::ApubCommunityOutbox,
  },
  http::{check_community_public, create_apub_response, create_apub_tombstone_response},
  objects::{community::ApubCommunity, person::ApubPerson},
};
use activitypub_federation::{
  actix_web::inbox::receive_activity,
  config::Data,
  protocol::context::WithContext,
  traits::{Collection, Object},
};
use actix_web::{web, web::Bytes, HttpRequest, HttpResponse};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{source::community::Community, traits::ApubActor};
use lemmy_utils::{error::LemmyResult, LemmyErrorType};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub(crate) struct CommunityQuery {
  community_name: String,
}

/// Return the ActivityPub json representation of a local community over HTTP.
#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_community_http(
  info: web::Path<CommunityQuery>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.community_name, true)
      .await?
      .ok_or(LemmyErrorType::CouldntFindCommunity)?
      .into();

  if community.deleted || community.removed {
    return create_apub_tombstone_response(community.actor_id.clone());
  }
  check_community_public(&community)?;

  let apub = community.into_json(&context).await?;
  create_apub_response(&apub)
}

/// Handler for all incoming receive to community inboxes.
#[tracing::instrument(skip_all)]
pub async fn community_inbox(
  request: HttpRequest,
  body: Bytes,
  data: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  receive_activity::<WithContext<GroupInboxActivities>, ApubPerson, LemmyContext>(
    request, body, &data,
  )
  .await
}

/// Returns an empty followers collection, only populating the size (for privacy).
pub(crate) async fn get_apub_community_followers(
  info: web::Path<CommunityQuery>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let community = Community::read_from_name(&mut context.pool(), &info.community_name, false)
    .await?
    .ok_or(LemmyErrorType::CouldntFindCommunity)?;
  check_community_public(&community)?;
  let followers = ApubCommunityFollower::read_local(&community.into(), &context).await?;
  create_apub_response(&followers)
}

/// Returns the community outbox, which is populated by a maximum of 20 posts (but no other
/// activities like votes or comments).
pub(crate) async fn get_apub_community_outbox(
  info: web::Path<CommunityQuery>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.community_name, false)
      .await?
      .ok_or(LemmyErrorType::CouldntFindCommunity)?
      .into();
  check_community_public(&community)?;
  let outbox = ApubCommunityOutbox::read_local(&community, &context).await?;
  create_apub_response(&outbox)
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_community_moderators(
  info: web::Path<CommunityQuery>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.community_name, false)
      .await?
      .ok_or(LemmyErrorType::CouldntFindCommunity)?
      .into();
  check_community_public(&community)?;
  let moderators = ApubCommunityModerators::read_local(&community, &context).await?;
  create_apub_response(&moderators)
}

/// Returns collection of featured (stickied) posts.
pub(crate) async fn get_apub_community_featured(
  info: web::Path<CommunityQuery>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.community_name, false)
      .await?
      .ok_or(LemmyErrorType::CouldntFindCommunity)?
      .into();
  check_community_public(&community)?;
  let featured = ApubCommunityFeatured::read_local(&community, &context).await?;
  create_apub_response(&featured)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
pub(crate) mod tests {

  use super::*;
  use crate::protocol::objects::{group::Group, tombstone::Tombstone};
  use actix_web::body::to_bytes;
  use lemmy_db_schema::{
    newtypes::InstanceId,
    source::{
      community::CommunityInsertForm,
      instance::Instance,
      local_site::{LocalSite, LocalSiteInsertForm},
      local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitInsertForm},
      site::{Site, SiteInsertForm},
    },
    traits::Crud,
    CommunityVisibility,
  };
  use serde::de::DeserializeOwned;
  use serial_test::serial;

  async fn init(
    deleted: bool,
    visibility: CommunityVisibility,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<(Instance, Community)> {
    let instance =
      Instance::read_or_create(&mut context.pool(), "my_domain.tld".to_string()).await?;
    create_local_site(context, instance.id).await?;

    let community_form = CommunityInsertForm::builder()
      .name("testcom6".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(instance.id)
      .deleted(Some(deleted))
      .visibility(Some(visibility))
      .build();
    let community = Community::create(&mut context.pool(), &community_form).await?;
    Ok((instance, community))
  }

  /// Necessary for the community outbox fetching
  async fn create_local_site(
    context: &Data<LemmyContext>,
    instance_id: InstanceId,
  ) -> LemmyResult<()> {
    // Create a local site, since this is necessary for community fetching.
    let site_form = SiteInsertForm::builder()
      .name("test site".to_string())
      .instance_id(instance_id)
      .build();
    let site = Site::create(&mut context.pool(), &site_form).await?;

    let local_site_form = LocalSiteInsertForm::builder().site_id(site.id).build();
    let local_site = LocalSite::create(&mut context.pool(), &local_site_form).await?;
    let local_site_rate_limit_form = LocalSiteRateLimitInsertForm::builder()
      .local_site_id(local_site.id)
      .build();

    LocalSiteRateLimit::create(&mut context.pool(), &local_site_rate_limit_form).await?;
    Ok(())
  }

  async fn decode_response<T: DeserializeOwned>(res: HttpResponse) -> LemmyResult<T> {
    let body = to_bytes(res.into_body()).await.unwrap();
    let body = std::str::from_utf8(&body)?;
    Ok(serde_json::from_str(body)?)
  }

  #[tokio::test]
  #[serial]
  async fn test_get_community() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let (instance, community) = init(false, CommunityVisibility::Public, &context).await?;

    // fetch invalid community
    let query = CommunityQuery {
      community_name: "asd".to_string(),
    };
    let res = get_apub_community_http(query.into(), context.reset_request_count()).await;
    assert!(res.is_err());

    // fetch valid community
    let query = CommunityQuery {
      community_name: community.name.clone(),
    };
    let res = get_apub_community_http(query.clone().into(), context.reset_request_count()).await?;
    assert_eq!(200, res.status());
    let res_group: Group = decode_response(res).await?;
    let community: ApubCommunity = community.into();
    let group = community.clone().into_json(&context).await?;
    assert_eq!(group, res_group);

    let res =
      get_apub_community_featured(query.clone().into(), context.reset_request_count()).await?;
    assert_eq!(200, res.status());
    let res =
      get_apub_community_followers(query.clone().into(), context.reset_request_count()).await?;
    assert_eq!(200, res.status());
    let res =
      get_apub_community_moderators(query.clone().into(), context.reset_request_count()).await?;
    assert_eq!(200, res.status());
    let res = get_apub_community_outbox(query.into(), context.reset_request_count()).await?;
    assert_eq!(200, res.status());

    Instance::delete(&mut context.pool(), instance.id).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_get_deleted_community() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let (instance, community) = init(true, CommunityVisibility::LocalOnly, &context).await?;

    // should return tombstone
    let query = CommunityQuery {
      community_name: community.name.clone(),
    };
    let res = get_apub_community_http(query.clone().into(), context.reset_request_count()).await?;
    assert_eq!(410, res.status());
    let res_tombstone = decode_response::<Tombstone>(res).await;
    assert!(res_tombstone.is_ok());

    let res =
      get_apub_community_featured(query.clone().into(), context.reset_request_count()).await;
    assert!(res.is_err());
    let res =
      get_apub_community_followers(query.clone().into(), context.reset_request_count()).await;
    assert!(res.is_err());
    let res =
      get_apub_community_moderators(query.clone().into(), context.reset_request_count()).await;
    assert!(res.is_err());
    let res = get_apub_community_outbox(query.into(), context.reset_request_count()).await;
    assert!(res.is_err());

    //Community::delete(&mut context.pool(), community.id).await?;
    Instance::delete(&mut context.pool(), instance.id).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_get_local_only_community() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let (instance, community) = init(false, CommunityVisibility::LocalOnly, &context).await?;

    let query = CommunityQuery {
      community_name: community.name.clone(),
    };
    let res = get_apub_community_http(query.clone().into(), context.reset_request_count()).await;
    assert!(res.is_err());
    let res =
      get_apub_community_featured(query.clone().into(), context.reset_request_count()).await;
    assert!(res.is_err());
    let res =
      get_apub_community_followers(query.clone().into(), context.reset_request_count()).await;
    assert!(res.is_err());
    let res =
      get_apub_community_moderators(query.clone().into(), context.reset_request_count()).await;
    assert!(res.is_err());
    let res = get_apub_community_outbox(query.into(), context.reset_request_count()).await;
    assert!(res.is_err());

    Instance::delete(&mut context.pool(), instance.id).await?;
    Ok(())
  }
}
