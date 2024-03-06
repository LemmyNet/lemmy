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
  utils::check_private_instance,
};
use lemmy_db_schema::{newtypes::PersonId, source::local_site::LocalSite, utils::DbPool};
use lemmy_db_views::structs::{CommentView, LocalUserView, PostView};
use lemmy_db_views_actor::structs::{CommunityView, PersonView};
use lemmy_utils::error::{LemmyError, LemmyErrorExt2, LemmyErrorType};
use crate::fetcher::user_or_community::UserOrCommunity;

#[tracing::instrument(skip(context))]
pub async fn resolve_object(
  data: Query<ResolveObject>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> Result<Json<ResolveObjectResponse>, LemmyError> {
  let local_site = LocalSite::read(&mut context.pool()).await?;
  check_private_instance(&local_user_view, &local_site)?;
  let person_id = local_user_view.map(|v| v.person.id);
  // If we get a valid personId back we can safely assume that the user is authenticated,
  // if there's no personId then the JWT was missing or invalid.
  let is_authenticated = person_id.is_some();

  let res = if is_authenticated {
    // user is fully authenticated; allow remote lookups as well.
    search_query_to_object_id(data.q.clone(), &context).await
  } else {
    // user isn't authenticated only allow a local search.
    search_query_to_object_id_local(&data.q, &context).await
  }
  .with_lemmy_type(LemmyErrorType::CouldntFindObject)?;

  convert_response(res, person_id, &mut context.pool())
    .await
    .with_lemmy_type(LemmyErrorType::CouldntFindObject)
}

async fn convert_response(
  object: SearchableObjects,
  user_id: Option<PersonId>,
  pool: &mut DbPool<'_>,
) -> Result<Json<ResolveObjectResponse>, LemmyError> {
  use SearchableObjects::*;
  let removed_or_deleted;
  let mut res = ResolveObjectResponse::default();
  match object {
    Post(p) => {
      removed_or_deleted = p.deleted || p.removed;
      res.post = Some(PostView::read(pool, p.id, user_id, false).await?)
    }
    Comment(c) => {
      removed_or_deleted = c.deleted || c.removed;
      res.comment = Some(CommentView::read(pool, c.id, user_id).await?)
    }
    PersonOrCommunity(p) => {
      match p {
        UserOrCommunity::User(u) => {
          removed_or_deleted = u.deleted;
          res.person = Some(PersonView::read(pool, u.id).await?)
        }
        UserOrCommunity::Community(c) => {
          removed_or_deleted = c.deleted || c.removed;
          res.community = Some(CommunityView::read(pool, c.id, user_id, false).await?)
        }
      }
    }
  };
  // if the object was deleted from database, dont return it
  if removed_or_deleted {
    Err(NotFound {}.into())
  } else {
    Ok(Json(res))
  }
}
