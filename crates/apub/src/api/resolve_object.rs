use crate::fetcher::{
  post_or_comment::PostOrComment,
  search::{search_query_to_object_id, search_query_to_object_id_local, SearchableObjects},
  user_or_community::UserOrCommunity,
};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  site::{ResolveObject, ResolveObjectResponse},
  utils::check_private_instance,
};
use lemmy_db_schema::utils::DbPool;
use lemmy_db_views::structs::{
  CommentView,
  CommunityView,
  LocalUserView,
  PersonView,
  PostView,
  SiteView,
};
use lemmy_utils::error::{LemmyErrorExt2, LemmyErrorType, LemmyResult};

pub async fn resolve_object(
  data: Query<ResolveObject>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<ResolveObjectResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  check_private_instance(&local_user_view, &local_site)?;
  // If we get a valid personId back we can safely assume that the user is authenticated,
  // if there's no personId then the JWT was missing or invalid.
  let is_authenticated = local_user_view.is_some();

  let res = if is_authenticated || cfg!(debug_assertions) {
    // user is fully authenticated; allow remote lookups as well.
    search_query_to_object_id(data.q.clone(), &context).await
  } else {
    // user isn't authenticated only allow a local search.
    search_query_to_object_id_local(&data.q, &context).await
  }
  .with_lemmy_type(LemmyErrorType::NotFound)?;

  convert_response(res, local_user_view, &mut context.pool())
    .await
    .with_lemmy_type(LemmyErrorType::NotFound)
}

async fn convert_response(
  object: SearchableObjects,
  local_user_view: Option<LocalUserView>,
  pool: &mut DbPool<'_>,
) -> LemmyResult<Json<ResolveObjectResponse>> {
  let mut res = ResolveObjectResponse::default();
  let local_user = local_user_view.map(|l| l.local_user);
  let is_admin = local_user.clone().map(|l| l.admin).unwrap_or_default();

  let site_view = SiteView::read_local(pool).await?;
  let local_instance_id = site_view.site.instance_id;

  match object {
    SearchableObjects::PostOrComment(pc) => match *pc {
      PostOrComment::Post(p) => {
        res.post =
          Some(PostView::read(pool, p.id, local_user.as_ref(), local_instance_id, is_admin).await?)
      }
      PostOrComment::Comment(c) => {
        res.comment =
          Some(CommentView::read(pool, c.id, local_user.as_ref(), local_instance_id).await?)
      }
    },
    SearchableObjects::PersonOrCommunity(pc) => match *pc {
      UserOrCommunity::User(u) => {
        res.person = Some(PersonView::read(pool, u.id, local_instance_id, is_admin).await?)
      }
      UserOrCommunity::Community(c) => {
        res.community = Some(CommunityView::read(pool, c.id, local_user.as_ref(), is_admin).await?)
      }
    },
  };

  Ok(Json(res))
}

#[cfg(test)]
mod tests {
  use crate::api::resolve_object::resolve_object;
  use actix_web::web::Query;
  use lemmy_api_common::{context::LemmyContext, site::ResolveObject};
  use lemmy_db_schema::{
    source::{
      community::{Community, CommunityInsertForm},
      instance::Instance,
      local_site::LocalSite,
      post::{Post, PostInsertForm, PostUpdateForm},
    },
    traits::Crud,
  };
  use lemmy_db_views::{site::site_view::create_test_instance, structs::LocalUserView};
  use lemmy_utils::error::{LemmyErrorType, LemmyResult};
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  #[expect(clippy::unwrap_used)]
  async fn test_object_visibility() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();
    let instance = create_test_instance(pool).await?;

    let name = "test_local_user_name";
    let bio = "test_local_user_bio";

    let creator = LocalUserView::create_test_user(pool, name, bio, false).await?;
    let regular_user = LocalUserView::create_test_user(pool, name, bio, false).await?;
    let admin_user = LocalUserView::create_test_user(pool, name, bio, true).await?;

    let community = Community::create(
      pool,
      &CommunityInsertForm::new(
        instance.id,
        "test".to_string(),
        "test".to_string(),
        "pubkey".to_string(),
      ),
    )
    .await?;

    let post_insert_form = PostInsertForm::new("Test".to_string(), creator.person.id, community.id);
    let post = Post::create(pool, &post_insert_form).await?;

    let query = format!("q={}", post.ap_id).to_string();
    let query: Query<ResolveObject> = Query::from_query(&query)?;

    // Objects should be resolvable without authentication
    let res = resolve_object(query.clone(), context.reset_request_count(), None).await?;
    assert_eq!(res.post.as_ref().unwrap().post.ap_id, post.ap_id);
    // Objects should be resolvable by regular users
    let res = resolve_object(
      query.clone(),
      context.reset_request_count(),
      Some(regular_user.clone()),
    )
    .await?;
    assert_eq!(res.post.as_ref().unwrap().post.ap_id, post.ap_id);
    // Objects should be resolvable by admins
    let res = resolve_object(
      query.clone(),
      context.reset_request_count(),
      Some(admin_user.clone()),
    )
    .await?;
    assert_eq!(res.post.as_ref().unwrap().post.ap_id, post.ap_id);

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
    let res = resolve_object(query.clone(), context.reset_request_count(), None).await;
    assert!(res.is_err_and(|e| e.error_type == LemmyErrorType::NotFound));
    // Deleted objects should not be resolvable by regular users
    let res = resolve_object(
      query.clone(),
      context.reset_request_count(),
      Some(regular_user.clone()),
    )
    .await;
    assert!(res.is_err_and(|e| e.error_type == LemmyErrorType::NotFound));
    // Deleted objects should be resolvable by admins
    let res = resolve_object(
      query.clone(),
      context.reset_request_count(),
      Some(admin_user.clone()),
    )
    .await?;
    assert_eq!(res.post.as_ref().unwrap().post.ap_id, post.ap_id);

    LocalSite::delete(pool).await?;
    Instance::delete(pool, instance.id).await?;

    Ok(())
  }
}
