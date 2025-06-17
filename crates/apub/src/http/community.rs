use super::check_community_content_fetchable;
use crate::{
  collections::{
    community_featured::ApubCommunityFeatured,
    community_follower::ApubCommunityFollower,
    community_moderators::ApubCommunityModerators,
    community_outbox::ApubCommunityOutbox,
  },
  fetcher::get_instance_id,
  http::{check_community_fetchable, create_apub_response, create_apub_tombstone_response},
};
use activitypub_federation::{
  actix_web::signing_actor,
  config::Data,
  fetch::object_id::ObjectId,
  traits::{Collection, Object},
};
use actix_web::{
  web::{Path, Query},
  HttpRequest,
  HttpResponse,
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::objects::{
  community::ApubCommunity,
  multi_community::ApubMultiCommunity,
  multi_community_collection::ApubFeedCollection,
  SiteOrMultiOrCommunityOrUser,
};
use lemmy_db_schema::{
  source::{community::Community, multi_community::MultiCommunity},
  traits::ApubActor,
};
use lemmy_db_schema_file::enums::CommunityVisibility;
use lemmy_db_views_community_follower::CommunityFollowerView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub(crate) struct CommunityPath {
  community_name: String,
}

#[derive(Deserialize, Clone)]
pub struct CommunityIsFollowerQuery {
  is_follower: Option<ObjectId<SiteOrMultiOrCommunityOrUser>>,
}

/// Return the ActivityPub json representation of a local community over HTTP.
pub(crate) async fn get_apub_community_http(
  info: Path<CommunityPath>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.community_name, true)
      .await?
      .ok_or(LemmyErrorType::NotFound)?
      .into();

  if community.deleted || community.removed {
    return create_apub_tombstone_response(community.ap_id.clone());
  }
  check_community_fetchable(&community)?;

  let apub = community.into_json(&context).await?;
  create_apub_response(&apub)
}

/// Returns an empty followers collection, only populating the size (for privacy).
pub(crate) async fn get_apub_community_followers(
  info: Path<CommunityPath>,
  query: Query<CommunityIsFollowerQuery>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let community = Community::read_from_name(&mut context.pool(), &info.community_name, false)
    .await?
    .ok_or(LemmyErrorType::NotFound)?;
  if let Some(is_follower) = &query.is_follower {
    return check_is_follower(community, is_follower, context, request).await;
  }
  check_community_fetchable(&community)?;
  let followers = ApubCommunityFollower::read_local(&community.into(), &context).await?;
  create_apub_response(&followers)
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
  CommunityFollowerView::check_has_followers_from_instance(
    community.id,
    get_instance_id(&signing_actor),
    &mut context.pool(),
  )
  .await?;

  let instance_id = get_instance_id(&is_follower.dereference(&context).await?);
  let has_followers = CommunityFollowerView::check_has_followers_from_instance(
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
    Community::read_from_name(&mut context.pool(), &info.community_name, false)
      .await?
      .ok_or(LemmyErrorType::NotFound)?
      .into();
  check_community_content_fetchable(&community, &request, &context).await?;
  let outbox = ApubCommunityOutbox::read_local(&community, &context).await?;
  create_apub_response(&outbox)
}

pub(crate) async fn get_apub_community_moderators(
  info: Path<CommunityPath>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.community_name, false)
      .await?
      .ok_or(LemmyErrorType::NotFound)?
      .into();
  check_community_fetchable(&community)?;
  let moderators = ApubCommunityModerators::read_local(&community, &context).await?;
  create_apub_response(&moderators)
}

/// Returns collection of featured (stickied) posts.
pub(crate) async fn get_apub_community_featured(
  info: Path<CommunityPath>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.community_name, false)
      .await?
      .ok_or(LemmyErrorType::NotFound)?
      .into();
  check_community_content_fetchable(&community, &request, &context).await?;
  let featured = ApubCommunityFeatured::read_local(&community, &context).await?;
  create_apub_response(&featured)
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
    MultiCommunity::read_from_name(&mut context.pool(), &query.multi_name)
      .await?
      .into();

  create_apub_response(&multi.into_json(&context).await?)
}

pub(crate) async fn get_apub_person_multi_community_follows(
  query: Path<MultiCommunityQuery>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let multi = MultiCommunity::read_from_name(&mut context.pool(), &query.multi_name)
    .await?
    .into();

  let collection = ApubFeedCollection::read_local(&multi, &context).await?;
  create_apub_response(&collection)
}

#[cfg(test)]
pub(crate) mod tests {

  use super::*;
  use actix_web::{body::to_bytes, test::TestRequest};
  use lemmy_apub_objects::protocol::{group::Group, tombstone::Tombstone};
  use lemmy_db_schema::{
    source::{
      community::CommunityInsertForm,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    test_data::TestData,
    traits::Crud,
  };
  use serde::de::DeserializeOwned;
  use serial_test::serial;

  async fn init(
    deleted: bool,
    visibility: CommunityVisibility,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<(TestData, Community, Path<CommunityPath>)> {
    let data = TestData::create(&mut context.pool()).await?;

    let community_form = CommunityInsertForm {
      deleted: Some(deleted),
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

    //Community::delete(&mut context.pool(), community.id).await?;
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
