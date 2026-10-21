use super::{ActorPath, check_community_content_fetchable};
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
pub(crate) struct CommunityIsFollowerQuery {
  is_follower: Option<ObjectId<SiteOrMultiOrCommunityOrUser>>,
}

/// Return the ActivityPub json representation of a community over HTTP.
pub(crate) async fn get_apub_community_http(
  info: Path<ActorPath>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let (name, domain) = info.split_name();
  let community: ApubCommunity = Community::read_from_name(&mut context.pool(), name, domain, true)
    .await?
    .ok_or(LemmyErrorType::NotFound)?
    .into();

  check_community_fetchable(&community)?;

  community.http_response(&FEDERATION_CONTEXT, &context).await
}

/// Returns an empty followers collection, only populating the size (for privacy).
pub(crate) async fn get_apub_community_followers(
  info: Path<ActorPath>,
  query: Query<CommunityIsFollowerQuery>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let community = Community::read_from_name(&mut context.pool(), &info.name, None, false)
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

#[derive(Deserialize, Clone, Default)]
pub(crate) struct CommunityOutboxQuery {
  /// If set, return an `OrderedCollectionPage` instead of the collection.
  /// `true` is the first page; any other value is a pagination cursor.
  page: Option<String>,
}

/// Returns the community outbox (posts only, not comments or votes).
///
/// Without query parameters this is an `OrderedCollection` with the first page of items
/// inlined (at most `FETCH_LIMIT_MAX`) plus a `first` link. `GET .../outbox?page=true`
/// (and later `?page={cursor}`) returns `OrderedCollectionPage`s with `next`/`prev`.
pub(crate) async fn get_apub_community_outbox(
  info: Path<ActorPath>,
  query: Query<CommunityOutboxQuery>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.name, None, false)
      .await?
      .ok_or(LemmyErrorType::NotFound)?
      .into();
  check_community_content_fetchable(&community, &request, &context).await?;
  if let Some(page) = &query.page {
    let outbox = ApubCommunityOutbox::read_local_page(&community, &context, page).await?;
    Ok(create_http_response(outbox, &FEDERATION_CONTEXT)?)
  } else {
    let outbox = ApubCommunityOutbox::read_local(&community, &context).await?;
    Ok(create_http_response(outbox, &FEDERATION_CONTEXT)?)
  }
}

pub(crate) async fn get_apub_community_moderators(
  info: Path<ActorPath>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.name, None, false)
      .await?
      .ok_or(LemmyErrorType::NotFound)?
      .into();
  check_community_fetchable(&community)?;
  let moderators = ApubCommunityModerators::read_local(&community, &context).await?;
  Ok(create_http_response(moderators, &FEDERATION_CONTEXT)?)
}

/// Returns collection of featured (stickied) posts.
pub(crate) async fn get_apub_community_featured(
  info: Path<ActorPath>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let community: ApubCommunity =
    Community::read_from_name(&mut context.pool(), &info.name, None, false)
      .await?
      .ok_or(LemmyErrorType::NotFound)?
      .into();
  check_community_content_fetchable(&community, &request, &context).await?;
  let featured = ApubCommunityFeatured::read_local(&community, &context).await?;
  Ok(create_http_response(featured, &FEDERATION_CONTEXT)?)
}

pub(crate) async fn get_apub_person_multi_community(
  path: Path<ActorPath>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let (name, domain) = path.split_name();
  let multi: ApubMultiCommunity =
    MultiCommunity::read_from_name(&mut context.pool(), name, domain, false)
      .await?
      .ok_or(LemmyErrorType::NotFound)?
      .into();

  multi.http_response(&FEDERATION_CONTEXT, &context).await
}

pub(crate) async fn get_apub_person_multi_community_follows(
  path: Path<ActorPath>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let multi = MultiCommunity::read_from_name(&mut context.pool(), &path.name, None, false)
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
  use crate::protocol::collections::group_outbox::{GroupOutbox, GroupOutboxPage};
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
    utils::FETCH_LIMIT_MAX,
  };
  use lemmy_diesel_utils::traits::Crud;
  use serde::de::DeserializeOwned;
  use serial_test::serial;
  use url::Url;

  async fn init(
    deleted: bool,
    visibility: CommunityVisibility,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<(TestData, Community, Path<ActorPath>)> {
    let data = TestData::create(&mut context.pool()).await?;

    let community_form = CommunityInsertForm {
      deleted: Some(deleted),
      ap_id: Some(Url::parse("http://lemmy-alpha")?.into()),
      visibility: Some(visibility),
      ..CommunityInsertForm::new(
        data.instance.id,
        "testcom6".to_string(),
        "pubkey".to_string(),
      )
    };
    let community = Community::create(&mut context.pool(), &community_form).await?;
    let path: Path<ActorPath> = ActorPath {
      name: community.name.clone(),
    }
    .into();
    Ok((data, community, path))
  }

  async fn decode_response<T: DeserializeOwned>(res: HttpResponse) -> LemmyResult<T> {
    let body = to_bytes(res.into_body()).await.unwrap_or_default();
    let body = std::str::from_utf8(&body)?;
    Ok(serde_json::from_str(body)?)
  }

  fn outbox_query(page: Option<&str>) -> Query<CommunityOutboxQuery> {
    Query(CommunityOutboxQuery {
      page: page.map(str::to_owned),
    })
  }

  fn page_query_value(url: &Url) -> Option<String> {
    url
      .query_pairs()
      .find(|(key, _)| key == "page")
      .map(|(_, value)| value.into_owned())
  }

  fn announce_ids(
    items: &[lemmy_apub_activities::protocol::community::announce::AnnounceActivity],
  ) -> Vec<String> {
    items
      .iter()
      .filter_map(|activity| {
        serde_json::to_value(activity)
          .ok()?
          .get("id")?
          .as_str()
          .map(str::to_owned)
      })
      .collect()
  }

  #[tokio::test]
  #[serial]
  async fn test_get_community() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let (data, community, path) = init(false, CommunityVisibility::Public, &context).await?;
    let request = TestRequest::default().to_http_request();

    // fetch invalid community
    let query = ActorPath {
      name: "asd".to_string(),
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
    let res = get_apub_community_outbox(path, outbox_query(None), context.clone(), request).await?;
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
    let res = get_apub_community_outbox(path, outbox_query(None), context.clone(), request).await;
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
    let res = get_apub_community_outbox(path, outbox_query(None), context.clone(), request).await;
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

    let res = get_apub_community_outbox(path, outbox_query(None), context.clone(), request).await?;
    assert_eq!(200, res.status());

    data.delete(&mut context.pool()).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_outbox_pagination() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let (data, community, path) = init(false, CommunityVisibility::Public, &context).await?;
    let request = TestRequest::default().to_http_request();
    let post_count = i32::try_from(FETCH_LIMIT_MAX)? + 1;

    for i in 0..post_count {
      let form = PostInsertForm::new(format!("post {i}"), data.person.id, community.id);
      Post::create(&mut context.pool(), &form).await?;
    }

    let res = get_apub_community_outbox(
      path.clone().into(),
      outbox_query(None),
      context.clone(),
      request.clone(),
    )
    .await?;
    assert_eq!(200, res.status());
    let collection: GroupOutbox = decode_response(res).await?;
    assert_eq!(post_count, collection.total_items);
    assert_eq!(FETCH_LIMIT_MAX, collection.ordered_items.len());
    assert_eq!(
      Some("true".to_string()),
      collection.first.as_ref().and_then(page_query_value)
    );

    let res = get_apub_community_outbox(
      path.clone().into(),
      outbox_query(Some("true")),
      context.clone(),
      request.clone(),
    )
    .await?;
    assert_eq!(200, res.status());
    let first_page: GroupOutboxPage = decode_response(res).await?;
    assert_eq!(collection.id, first_page.part_of);
    assert_eq!(post_count, first_page.total_items);
    assert_eq!(FETCH_LIMIT_MAX, first_page.ordered_items.len());
    assert_eq!(None, first_page.prev);
    let next_page_param = first_page
      .next
      .as_ref()
      .and_then(page_query_value)
      .ok_or(LemmyErrorType::NotFound)?;
    assert_ne!("true", next_page_param);
    assert_eq!(
      announce_ids(&collection.ordered_items),
      announce_ids(&first_page.ordered_items),
    );

    let res = get_apub_community_outbox(
      path,
      outbox_query(Some(&next_page_param)),
      context.clone(),
      request,
    )
    .await?;
    assert_eq!(200, res.status());
    let second_page: GroupOutboxPage = decode_response(res).await?;
    assert_eq!(collection.id, second_page.part_of);
    assert_eq!(1, second_page.ordered_items.len());
    assert!(second_page.next.is_none());
    assert!(second_page.prev.is_some());
    assert_ne!(
      announce_ids(&first_page.ordered_items),
      announce_ids(&second_page.ordered_items)
    );

    data.delete(&mut context.pool()).await?;
    Ok(())
  }
}
