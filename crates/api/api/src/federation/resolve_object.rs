use activitypub_federation::{
  config::Data,
  fetch::{object_id::ObjectId, webfinger::webfinger_resolve_actor},
};
use actix_web::web::{Json, Query};
use either::Either::*;
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{check_is_mod_or_admin, check_private_instance},
};
use lemmy_apub_objects::objects::{SearchableObjects, UserOrCommunity};
use lemmy_db_schema_file::PersonId;
use lemmy_db_views_comment::CommentView;
use lemmy_db_views_community::{CommunityView, MultiCommunityView};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::PersonView;
use lemmy_db_views_post::PostView;
use lemmy_db_views_search_combined::{SearchCombinedView, SearchResponse};
use lemmy_db_views_site::{SiteView, api::ResolveObject};
use lemmy_utils::error::{LemmyErrorExt2, LemmyErrorType, LemmyResult};
use url::Url;

pub async fn resolve_object(
  data: Query<ResolveObject>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<SearchResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  check_private_instance(&local_user_view, &local_site)?;

  let res = resolve_object_internal(&data.q, &local_user_view, &context).await?;
  Ok(Json(SearchResponse {
    results: vec![res],
    ..Default::default()
  }))
}

pub(super) async fn resolve_object_internal(
  query: &str,
  local_user_view: &Option<LocalUserView>,
  context: &Data<LemmyContext>,
) -> LemmyResult<SearchCombinedView> {
  use SearchCombinedView::*;

  let is_authenticated = local_user_view.as_ref().is_some_and(|l| !l.banned);

  let object = if is_authenticated || cfg!(debug_assertions) {
    // user is fully authenticated; allow remote lookups as well.
    search_query_to_object_id(query.to_string(), context).await
  } else {
    // user isn't authenticated only allow a local search.
    search_query_to_object_id_local(query, context).await
  }
  .with_lemmy_type(LemmyErrorType::NotFound)?;

  let my_person_id_opt = local_user_view.as_ref().map(|l| l.person.id);
  let my_person_id = my_person_id_opt.unwrap_or(PersonId(-1));
  let local_user = local_user_view.as_ref().map(|l| l.local_user.clone());
  let is_admin = local_user.as_ref().map(|l| l.admin).unwrap_or_default();
  let pool = &mut context.pool();
  let local_instance_id = SiteView::read_local(pool).await?.site.instance_id;

  Ok(match object {
    Left(Left(Left(p))) => {
      let is_mod = check_is_mod_or_admin(pool, my_person_id, p.community_id)
        .await
        .is_ok();
      Post(PostView::read(pool, p.id, local_user.as_ref(), local_instance_id, is_mod).await?)
    }
    Left(Left(Right(c))) => {
      Comment(CommentView::read(pool, c.id, local_user.as_ref(), local_instance_id).await?)
    }
    Left(Right(Left(u))) => {
      Person(PersonView::read(pool, u.id, my_person_id_opt, local_instance_id, is_admin).await?)
    }
    Left(Right(Right(c))) => {
      Community(CommunityView::read(pool, c.id, local_user.as_ref(), is_admin).await?)
    }
    Right(multi) => {
      MultiCommunity(MultiCommunityView::read(pool, multi.id, my_person_id_opt).await?)
    }
  })
}

/// Converts search query to object id. The query can either be an URL, which will be treated as
/// ObjectId directly, or a webfinger identifier (@user@example.com or !community@example.com)
/// which gets resolved to an URL.
async fn search_query_to_object_id(
  mut query: String,
  context: &Data<LemmyContext>,
) -> LemmyResult<SearchableObjects> {
  Ok(match Url::parse(&query) {
    Ok(url) => {
      // its already an url, just go with it
      ObjectId::from(url).dereference(context).await?
    }
    Err(_) => {
      // not an url, try to resolve via webfinger
      if query.starts_with('!') || query.starts_with('@') {
        query.remove(0);
      }
      Left(Right(
        webfinger_resolve_actor::<LemmyContext, UserOrCommunity>(&query, context).await?,
      ))
    }
  })
}

/// Converts a search query to an object id.  The query MUST bbe a URL which will bbe treated
/// as the ObjectId directly.  If the query is a webfinger identifier (@user@example.com or
/// !community@example.com) this method will return an error.
async fn search_query_to_object_id_local(
  query: &str,
  context: &Data<LemmyContext>,
) -> LemmyResult<SearchableObjects> {
  let url = Url::parse(query)?;
  ObjectId::from(url).dereference_local(context).await
}

#[cfg(test)]
mod tests {
  use super::*;
  use lemmy_db_schema::{
    source::{
      community::{Community, CommunityInsertForm},
      local_site::LocalSite,
      post::{Post, PostInsertForm, PostUpdateForm},
    },
    test_data::TestData,
  };
  use lemmy_diesel_utils::traits::Crud;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_object_visibility() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();
    let data = TestData::create(pool).await?;

    let bio = "test_local_user_bio";

    let creator =
      LocalUserView::create_test_user(pool, "test_local_user_name_1", bio, false).await?;
    let regular_user =
      LocalUserView::create_test_user(pool, "test_local_user_name_2", bio, false).await?;
    let admin_user =
      LocalUserView::create_test_user(pool, "test_local_user_name_3", bio, true).await?;

    let community = Community::create(
      pool,
      &CommunityInsertForm::new(
        data.instance.id,
        "test".to_string(),
        "test".to_string(),
        "pubkey".to_string(),
      ),
    )
    .await?;

    let post_insert_form = PostInsertForm::new("Test".to_string(), creator.person.id, community.id);
    let post = Post::create(pool, &post_insert_form).await?;

    let query = post.ap_id.to_string();

    // Objects should be resolvable without authentication
    let res = resolve_object_internal(&query, &None, &context).await?;
    assert_response(res, &post);
    // Objects should be resolvable by regular users
    let res = resolve_object_internal(&query, &Some(regular_user.clone()), &context).await?;
    assert_response(res, &post);
    // Objects should be resolvable by admins
    let res = resolve_object_internal(&query, &Some(admin_user.clone()), &context).await?;
    assert_response(res, &post);

    Post::update(
      pool,
      post.id,
      &PostUpdateForm {
        deleted: Some(true),
        ..Default::default()
      },
    )
    .await?;

    // Deleted objects should not be resolvable without authentication
    let res = resolve_object_internal(&query, &None, &context).await;
    assert!(res.is_err_and(|e| e.error_type == LemmyErrorType::NotFound));
    // Deleted objects should not be resolvable by regular users
    let res = resolve_object_internal(&query, &Some(regular_user.clone()), &context).await;
    assert!(res.is_err_and(|e| e.error_type == LemmyErrorType::NotFound));
    // Deleted objects should be resolvable by admins
    let res = resolve_object_internal(&query, &Some(admin_user.clone()), &context).await?;
    assert_response(res, &post);

    LocalSite::delete(pool).await?;
    data.delete(pool).await?;

    Ok(())
  }

  fn assert_response(res: SearchCombinedView, expected_post: &Post) {
    if let SearchCombinedView::Post(v) = res {
      assert_eq!(expected_post.ap_id, v.post.ap_id);
    } else {
      panic!("invalid resolve object response");
    }
  }
}
