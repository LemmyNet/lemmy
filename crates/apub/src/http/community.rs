use crate::{
  activity_lists::GroupInboxActivities,
  collections::{
    community_featured::ApubCommunityFeatured,
    community_follower::ApubCommunityFollower,
    community_moderators::ApubCommunityModerators,
    community_outbox::ApubCommunityOutbox,
  },
  http::{create_apub_response, create_apub_tombstone_response},
  objects::{community::ApubCommunity, person::ApubPerson},
};
use activitypub_federation::{
  actix_web::inbox::receive_activity,
  config::Data,
  protocol::context::WithContext,
  traits::{Collection, Object},
};
use actix_web::{web, web::Bytes, HttpRequest, HttpResponse};
use lemmy_api_common::{context::LemmyContext, utils::check_community_valid};
use lemmy_db_schema::{source::community::Community, traits::ApubActor};
use lemmy_utils::error::LemmyError;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct CommunityQuery {
  community_name: String,
}

/// Return the ActivityPub json representation of a local community over HTTP.
#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_community_http(
  info: web::Path<CommunityQuery>,
  context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.community_name, true)
      .await?
      .into();

  if community.deleted || community.removed {
    return create_apub_tombstone_response(community.actor_id.clone());
  }
  check_community_valid(&community)?;

  let apub = community.into_json(&context).await?;
  create_apub_response(&apub)
}

/// Handler for all incoming receive to community inboxes.
#[tracing::instrument(skip_all)]
pub async fn community_inbox(
  request: HttpRequest,
  body: Bytes,
  data: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  receive_activity::<WithContext<GroupInboxActivities>, ApubPerson, LemmyContext>(
    request, body, &data,
  )
  .await
}

/// Returns an empty followers collection, only populating the size (for privacy).
pub(crate) async fn get_apub_community_followers(
  info: web::Path<CommunityQuery>,
  context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let community =
    Community::read_from_name(&mut context.pool(), &info.community_name, false).await?;
  check_community_valid(&community)?;
  let followers = ApubCommunityFollower::read_local(&community.into(), &context).await?;
  create_apub_response(&followers)
}

/// Returns the community outbox, which is populated by a maximum of 20 posts (but no other
/// activites like votes or comments).
pub(crate) async fn get_apub_community_outbox(
  info: web::Path<CommunityQuery>,
  context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.community_name, false)
      .await?
      .into();
  check_community_valid(&community)?;
  let outbox = ApubCommunityOutbox::read_local(&community, &context).await?;
  create_apub_response(&outbox)
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_community_moderators(
  info: web::Path<CommunityQuery>,
  context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.community_name, false)
      .await?
      .into();
  check_community_valid(&community)?;
  let moderators = ApubCommunityModerators::read_local(&community, &context).await?;
  create_apub_response(&moderators)
}

/// Returns collection of featured (stickied) posts.
pub(crate) async fn get_apub_community_featured(
  info: web::Path<CommunityQuery>,
  context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.community_name, false)
      .await?
      .into();
  check_community_valid(&community)?;
  let featured = ApubCommunityFeatured::read_local(&community, &context).await?;
  create_apub_response(&featured)
}

#[cfg(test)]
pub(crate) mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use super::*;
  use crate::{objects::tests::init_context, protocol::objects::group::Group};
  use actix_web::body::to_bytes;
  use lemmy_db_schema::{
    source::{
      community::{Community, CommunityInsertForm},
      instance::Instance,
    },
    traits::Crud,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_get_community() {
    let context = init_context().await;

    // fetch invalid community
    let query = CommunityQuery {
      community_name: "asd".to_string(),
    };
    let res = get_apub_community_http(query.into(), context.reset_request_count()).await;
    assert!(res.is_err());

    let inserted_instance =
      Instance::read_or_create(&mut context.pool(), "my_domain.tld".to_string())
        .await
        .unwrap();
    let community_form = CommunityInsertForm::builder()
      .name("testcom5".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let community: ApubCommunity = Community::create(&mut context.pool(), &community_form)
      .await
      .unwrap()
      .into();
    let group = community.clone().into_json(&context).await.unwrap();

    // fetch valid community
    let query = CommunityQuery {
      community_name: community.name.clone(),
    };
    let res = get_apub_community_http(query.into(), context.reset_request_count())
      .await
      .unwrap();
    assert_eq!(200, res.status());
    let body = to_bytes(res.into_body()).await.unwrap();
    let body = std::str::from_utf8(&body).unwrap();
    let res_group: Group = serde_json::from_str(body).unwrap();
    assert_eq!(group, res_group);
  }

  #[tokio::test]
  #[serial]
  async fn test_get_deleted_community() {
    let context = init_context().await;

    // TODO: test that deleted community returns tombstone, also test local-only
  }
}
