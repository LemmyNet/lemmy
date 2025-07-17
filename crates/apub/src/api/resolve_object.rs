use crate::fetcher::search::{search_query_to_object_id, search_query_to_object_id_local};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use either::Either::*;
use lemmy_api_utils::{context::LemmyContext, utils::check_private_instance};
use lemmy_db_views_comment::CommentView;
use lemmy_db_views_community::{CommunityView, MultiCommunityView};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::PersonView;
use lemmy_db_views_post::PostView;
use lemmy_db_views_search_combined::{SearchCombinedView, SearchResponse};
use lemmy_db_views_site::{api::ResolveObject, SiteView};
use lemmy_utils::error::{LemmyErrorExt2, LemmyErrorType, LemmyResult};

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

  // If we get a valid personId back we can safely assume that the user is authenticated,
  // if there's no personId then the JWT was missing or invalid.
  let is_authenticated = local_user_view.is_some();

  let object = if is_authenticated || cfg!(debug_assertions) {
    // user is fully authenticated; allow remote lookups as well.
    search_query_to_object_id(query.to_string(), context).await
  } else {
    // user isn't authenticated only allow a local search.
    search_query_to_object_id_local(query, context).await
  }
  .with_lemmy_type(LemmyErrorType::NotFound)?;

  let my_person_id = local_user_view.as_ref().map(|l| l.person.id);
  let local_user = local_user_view.as_ref().map(|l| l.local_user.clone());
  let is_admin = local_user.as_ref().map(|l| l.admin).unwrap_or_default();
  let pool = &mut context.pool();
  let local_instance_id = SiteView::read_local(pool).await?.site.instance_id;

  Ok(match object {
    Left(Left(Left(p))) => {
      Post(PostView::read(pool, p.id, local_user.as_ref(), local_instance_id, is_admin).await?)
    }
    Left(Left(Right(c))) => {
      Comment(CommentView::read(pool, c.id, local_user.as_ref(), local_instance_id).await?)
    }
    Left(Right(Left(u))) => {
      Person(PersonView::read(pool, u.id, my_person_id, local_instance_id, is_admin).await?)
    }
    Left(Right(Right(c))) => {
      Community(CommunityView::read(pool, c.id, local_user.as_ref(), is_admin).await?)
    }
    Right(multi) => MultiCommunity(MultiCommunityView::read(pool, multi.id).await?),
  })
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
    traits::Crud,
  };
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
