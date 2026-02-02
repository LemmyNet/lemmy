use super::check_community_content_fetchable;
use crate::{
  collections::{
    community_featured::ApubCommunityFeatured,
    community_follower::ApubCommunityFollower,
    community_moderators::ApubCommunityModerators,
    community_outbox::ApubCommunityOutbox,
  },
  http::{check_community_fetchable, get_instance_id},
};
use activitypub_federation::{
  actix_web::{response::create_http_response, signing_actor},
  config::Data,
  fetch::object_id::ObjectId,
  traits::{Collection, Object},
};
use actix_web::{
  HttpRequest,
  HttpResponse,
  web::{Path, Query},
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{
    SiteOrMultiOrCommunityOrUser,
    community::ApubCommunity,
    multi_community::ApubMultiCommunity,
    multi_community_collection::ApubFeedCollection,
  },
  protocol::tags::ApubCommunityTag,
};
use lemmy_db_schema::{
  source::{community::Community, community_tag::CommunityTag, multi_community::MultiCommunity},
  traits::ApubActor,
};
use lemmy_db_schema_file::enums::CommunityVisibility;
use lemmy_db_views_community_follower_approval::PendingFollowerView;
use lemmy_utils::{
  FEDERATION_CONTEXT,
  error::{LemmyErrorType, LemmyResult},
};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub(crate) struct CommunityPath {
  community_name: String,
}

#[derive(Deserialize, Clone)]
pub(crate) struct CommunityIsFollowerQuery {
  is_follower: Option<ObjectId<SiteOrMultiOrCommunityOrUser>>,
}

/// Return the ActivityPub json representation of a local community over HTTP.
pub(crate) async fn get_apub_community_http(
  info: Path<CommunityPath>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.community_name, None, true)
      .await?
      .ok_or(LemmyErrorType::NotFound)?
      .into();

  check_community_fetchable(&community)?;

  community.http_response(&FEDERATION_CONTEXT, &context).await
}

/// Returns an empty followers collection, only populating the size (for privacy).
pub(crate) async fn get_apub_community_followers(
  info: Path<CommunityPath>,
  query: Query<CommunityIsFollowerQuery>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let community = Community::read_from_name(&mut context.pool(), &info.community_name, None, false)
    .await?
    .ok_or(LemmyErrorType::NotFound)?;
  if let Some(is_follower) = &query.is_follower {
    return check_is_follower(community, is_follower, context, request).await;
  }
  check_community_fetchable(&community)?;
  let followers = ApubCommunityFollower::read_local(&community.into(), &context).await?;
  Ok(create_http_response(followers, &FEDERATION_CONTEXT)?)
}

/// Checks if a given actor follows the private community. Returns status 200 if true.
async fn check_is_follower(
  community: Community,
  is_follower: &ObjectId<SiteOrMultiOrCommunityOrUser>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  if community.visibility != CommunityVisibility::Private {
    return Ok(HttpResponse::BadRequest().body("must be a private community"));
  }
  // also check for http sig so that followers are not exposed publicly
  let signing_actor =
    signing_actor::<SiteOrMultiOrCommunityOrUser>(&request, None, &context).await?;
  PendingFollowerView::check_has_followers_from_instance(
    community.id,
    get_instance_id(&signing_actor),
    &mut context.pool(),
  )
  .await?;

  let instance_id = get_instance_id(&is_follower.dereference(&context).await?);
  let has_followers = PendingFollowerView::check_has_followers_from_instance(
    community.id,
    instance_id,
    &mut context.pool(),
  )
  .await;
  if has_followers.is_ok() {
    Ok(HttpResponse::Ok().finish())
  } else {
    Ok(HttpResponse::NotFound().finish())
  }
}

/// Returns the community outbox, which is populated by a maximum of 20 posts (but no other
/// activities like votes or comments).
pub(crate) async fn get_apub_community_outbox(
  info: Path<CommunityPath>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.community_name, None, false)
      .await?
      .ok_or(LemmyErrorType::NotFound)?
      .into();
  check_community_content_fetchable(&community, &request, &context).await?;
  let outbox = ApubCommunityOutbox::read_local(&community, &context).await?;
  Ok(create_http_response(outbox, &FEDERATION_CONTEXT)?)
}

pub(crate) async fn get_apub_community_moderators(
  info: Path<CommunityPath>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.community_name, None, false)
      .await?
      .ok_or(LemmyErrorType::NotFound)?
      .into();
  check_community_fetchable(&community)?;
  let moderators = ApubCommunityModerators::read_local(&community, &context).await?;
  Ok(create_http_response(moderators, &FEDERATION_CONTEXT)?)
}

/// Returns collection of featured (stickied) posts.
pub(crate) async fn get_apub_community_featured(
  info: Path<CommunityPath>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.community_name, None, false)
      .await?
      .ok_or(LemmyErrorType::NotFound)?
      .into();
  check_community_content_fetchable(&community, &request, &context).await?;
  let featured = ApubCommunityFeatured::read_local(&community, &context).await?;
  Ok(create_http_response(featured, &FEDERATION_CONTEXT)?)
}

#[derive(Deserialize)]
pub(crate) struct MultiCommunityQuery {
  multi_name: String,
}

pub(crate) async fn get_apub_person_multi_community(
  query: Path<MultiCommunityQuery>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let multi: ApubMultiCommunity =
    MultiCommunity::read_from_name(&mut context.pool(), &query.multi_name, None, false)
      .await?
      .ok_or(LemmyErrorType::NotFound)?
      .into();

  multi.http_response(&FEDERATION_CONTEXT, &context).await
}

pub(crate) async fn get_apub_person_multi_community_follows(
  query: Path<MultiCommunityQuery>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let multi = MultiCommunity::read_from_name(&mut context.pool(), &query.multi_name, None, false)
    .await?
    .ok_or(LemmyErrorType::NotFound)?
    .into();

  let collection = ApubFeedCollection::read_local(&multi, &context).await?;
  Ok(create_http_response(collection, &FEDERATION_CONTEXT)?)
}

#[derive(Deserialize, Clone)]
pub(crate) struct CommunityTagPath {
  community_name: String,
  tag_name: String,
}

/// Return the ActivityPub json representation of a local community over HTTP.
pub(crate) async fn get_apub_community_tag_http(
  info: Path<CommunityTagPath>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.community_name, None, true)
      .await?
      .ok_or(LemmyErrorType::NotFound)?
      .into();

  check_community_fetchable(&community)?;

  let tag = CommunityTag::read_for_community(&mut context.pool(), community.id)
    .await?
    .into_iter()
    .map(ApubCommunityTag::to_json)
    .find(|t| t.preferred_username == info.tag_name)
    .ok_or(LemmyErrorType::NotFound)?;

  Ok(create_http_response(tag, &FEDERATION_CONTEXT)?)
}

#[cfg(test)]
pub(crate) mod tests {

  use super::*;
  use activitypub_federation::protocol::tombstone::Tombstone;
  use actix_web::{body::to_bytes, test::TestRequest};
  use lemmy_apub_objects::protocol::group::Group;
  use lemmy_db_schema::{
    source::{
      community::CommunityInsertForm,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    test_data::TestData,
  };
  use lemmy_diesel_utils::traits::Crud;
  use serde::de::DeserializeOwned;
  use serial_test::serial;
  use url::Url;

  async fn init(
    deleted: bool,
    visibility: CommunityVisibility,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<(TestData, Community, Path<CommunityPath>)> {
    let data = TestData::create(&mut context.pool()).await?;

    let community_form = CommunityInsertForm {
      deleted: Some(deleted),
      ap_id: Some(Url::parse("http://lemmy-alpha")?.into()),
      visibility: Some(visibility),
      ..CommunityInsertForm::new(
        data.instance.id,
        "testcom6".to_string(),
        "nada".to_owned(),
        "pubkey".to_string(),
      )
    };
    let community = Community::create(&mut context.pool(), &community_form).await?;
    let path: Path<CommunityPath> = CommunityPath {
      community_name: community.name.clone(),
    }
    .into();
    Ok((data, community, path))
  }

  async fn decode_response<T: DeserializeOwned>(res: HttpResponse) -> LemmyResult<T> {
    let body = to_bytes(res.into_body()).await.unwrap_or_default();
    let body = std::str::from_utf8(&body)?;
    Ok(serde_json::from_str(body)?)
  }

  #[tokio::test]
  #[serial]
  async fn test_get_community() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let (data, community, path) = init(false, CommunityVisibility::Public, &context).await?;
    let request = TestRequest::default().to_http_request();

    // fetch invalid community
    let query = CommunityPath {
      community_name: "asd".to_string(),
    };
    let res = get_apub_community_http(query.into(), context.clone()).await;
    assert!(res.is_err());

    // fetch valid community
    let res = get_apub_community_http(path.clone().into(), context.clone()).await?;
    assert_eq!(200, res.status());
    let res_group: Group = decode_response(res).await?;
    let community: ApubCommunity = community.into();
    let group = community.clone().into_json(&context).await?;
    assert_eq!(group, res_group);

    let res =
      get_apub_community_featured(path.clone().into(), context.clone(), request.clone()).await?;
    assert_eq!(200, res.status());
    let query = Query(CommunityIsFollowerQuery { is_follower: None });
    let res =
      get_apub_community_followers(path.clone().into(), query, context.clone(), request.clone())
        .await?;
    assert_eq!(200, res.status());
    let res = get_apub_community_moderators(path.clone().into(), context.clone()).await?;
    assert_eq!(200, res.status());
    let res = get_apub_community_outbox(path, context.clone(), request).await?;
    assert_eq!(200, res.status());

    data.delete(&mut context.pool()).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_get_deleted_community() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let (data, _, path) = init(true, CommunityVisibility::Public, &context).await?;
    let request = TestRequest::default().to_http_request();

    // should return tombstone
    let res = get_apub_community_http(path.clone().into(), context.clone()).await?;
    assert_eq!(410, res.status());
    let res_tombstone = decode_response::<Tombstone>(res).await;
    assert!(res_tombstone.is_ok());

    let res =
      get_apub_community_featured(path.clone().into(), context.clone(), request.clone()).await;
    assert!(res.is_err());
    let query = Query(CommunityIsFollowerQuery { is_follower: None });
    let res =
      get_apub_community_followers(path.clone().into(), query, context.clone(), request.clone())
        .await;
    assert!(res.is_err());
    let res = get_apub_community_moderators(path.clone().into(), context.clone()).await;
    assert!(res.is_err());
    let res = get_apub_community_outbox(path, context.clone(), request).await;
    assert!(res.is_err());

    data.delete(&mut context.pool()).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_get_local_only_community() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let (data, _, path) = init(false, CommunityVisibility::LocalOnlyPrivate, &context).await?;
    let request = TestRequest::default().to_http_request();

    let res = get_apub_community_http(path.clone().into(), context.clone()).await;
    assert!(res.is_err());
    let res =
      get_apub_community_featured(path.clone().into(), context.clone(), request.clone()).await;
    assert!(res.is_err());
    let query = Query(CommunityIsFollowerQuery { is_follower: None });
    let res =
      get_apub_community_followers(path.clone().into(), query, context.clone(), request.clone())
        .await;
    assert!(res.is_err());
    let res = get_apub_community_moderators(path.clone().into(), context.clone()).await;
    assert!(res.is_err());
    let res = get_apub_community_outbox(path, context.clone(), request).await;
    assert!(res.is_err());

    data.delete(&mut context.pool()).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_outbox_deleted_user() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let (data, community, path) = init(false, CommunityVisibility::Public, &context).await?;
    let request = TestRequest::default().to_http_request();

    // post from deleted user shouldnt break outbox
    let mut form = PersonInsertForm::new("jerry".to_string(), String::new(), data.instance.id);
    form.deleted = Some(true);
    let person = Person::create(&mut context.pool(), &form).await?;

    let form = PostInsertForm::new("title".to_string(), person.id, community.id);
    Post::create(&mut context.pool(), &form).await?;

    let res = get_apub_community_outbox(path, context.clone(), request).await?;
    assert_eq!(200, res.status());

    data.delete(&mut context.pool()).await?;
    Ok(())
  }
}
