use crate::fetcher::search::{
  search_query_to_object_id,
  search_query_to_object_id_local,
  SearchableObjects,
};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use diesel::NotFound;
use lemmy_api_common::{
  context::LemmyContext,
  site::{ResolveObject, ResolveObjectResponse},
  utils::{check_private_instance, local_user_view_from_jwt_opt},
};
use lemmy_db_schema::{newtypes::PersonId, source::local_site::LocalSite, utils::DbPool};
use lemmy_db_views::structs::{CommentView, PostView};
use lemmy_db_views_actor::structs::{CommunityView, PersonView};
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn resolve_object(
  data: Query<ResolveObject>,
  context: Data<LemmyContext>,
) -> Result<Json<ResolveObjectResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt_opt(data.auth.as_ref(), &context).await;
  let local_site = LocalSite::read(&mut context.pool()).await?;
  check_private_instance(&local_user_view, &local_site)?;
  let person_id = local_user_view.map(|v| v.person.id);
  // If we get a valid personId back we can safely assume that the user is authenticated,
  // if there's no personId then the JWT was missing or invalid.
  let is_authenticated = person_id.is_some();

  let res = if is_authenticated {
    // user is fully authenticated; allow remote lookups as well.
    search_query_to_object_id(&data.q, &context).await
  } else {
    // user isn't authenticated only allow a local search.
    search_query_to_object_id_local(&data.q, &context).await
  }
  .map_err(|e| e.with_message("couldnt_find_object"))?;

  convert_response(res, person_id, &mut context.pool())
    .await
    .map_err(|e| e.with_message("couldnt_find_object"))
}

async fn convert_response(
  object: SearchableObjects,
  user_id: Option<PersonId>,
  pool: &DbPool,
) -> Result<Json<ResolveObjectResponse>, LemmyError> {
  use SearchableObjects::*;
  let removed_or_deleted;
  let mut res = ResolveObjectResponse::default();
  match object {
    Person(p) => {
      removed_or_deleted = p.deleted;
      res.person = Some(PersonView::read(pool, p.id).await?)
    }
    Community(c) => {
      removed_or_deleted = c.deleted || c.removed;
      res.community = Some(CommunityView::read(pool, c.id, user_id, None).await?)
    }
    Post(p) => {
      removed_or_deleted = p.deleted || p.removed;
      res.post = Some(PostView::read(pool, p.id, user_id, None).await?)
    }
    Comment(c) => {
      removed_or_deleted = c.deleted || c.removed;
      res.comment = Some(CommentView::read(pool, c.id, user_id).await?)
    }
  };
  // if the object was deleted from database, dont return it
  if removed_or_deleted {
    return Err(NotFound {}.into());
  }
  Ok(Json(res))
}
