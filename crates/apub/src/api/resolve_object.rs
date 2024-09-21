use crate::fetcher::{
  search::{search_query_to_object_id, search_query_to_object_id_local, SearchableObjects},
  user_or_community::UserOrCommunity,
};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use diesel::NotFound;
use lemmy_api_common::{
  context::LemmyContext,
  site::{ResolveObject, ResolveObjectResponse},
  utils::check_private_instance,
};
use lemmy_db_schema::{source::local_site::LocalSite, utils::DbPool};
use lemmy_db_views::structs::{CommentView, LocalUserView, PostView};
use lemmy_db_views_actor::structs::{CommunityView, PersonView};
use lemmy_utils::error::{LemmyErrorExt2, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn resolve_object(
  data: Query<ResolveObject>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<ResolveObjectResponse>> {
  let local_site = LocalSite::read(&mut context.pool()).await?;
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
  .with_lemmy_type(LemmyErrorType::CouldntFindObject)?;

  convert_response(res, local_user_view, &mut context.pool())
    .await
    .with_lemmy_type(LemmyErrorType::CouldntFindObject)
}

async fn convert_response(
  object: SearchableObjects,
  local_user_view: Option<LocalUserView>,
  pool: &mut DbPool<'_>,
) -> LemmyResult<Json<ResolveObjectResponse>> {
  use SearchableObjects::*;
  let removed_or_deleted;
  let mut res = ResolveObjectResponse::default();
  let local_user = local_user_view.map(|l| l.local_user);

  match object {
    Post(p) => {
      removed_or_deleted = p.deleted || p.removed;
      res.post = Some(
        PostView::read(pool, p.id, local_user.as_ref(), false)
          .await?
          .ok_or(LemmyErrorType::CouldntFindPost)?,
      )
    }
    Comment(c) => {
      removed_or_deleted = c.deleted || c.removed;
      res.comment = Some(
        CommentView::read(pool, c.id, local_user.as_ref())
          .await?
          .ok_or(LemmyErrorType::CouldntFindComment)?,
      )
    }
    PersonOrCommunity(p) => match *p {
      UserOrCommunity::User(u) => {
        removed_or_deleted = u.deleted;
        res.person = Some(
          PersonView::read(pool, u.id)
            .await?
            .ok_or(LemmyErrorType::CouldntFindPerson)?,
        )
      }
      UserOrCommunity::Community(c) => {
        removed_or_deleted = c.deleted || c.removed;
        res.community = Some(
          CommunityView::read(pool, c.id, local_user.as_ref(), false)
            .await?
            .ok_or(LemmyErrorType::CouldntFindCommunity)?,
        )
      }
    },
  };
  // if the object was deleted from database, dont return it
  if removed_or_deleted {
    Err(NotFound {}.into())
  } else {
    Ok(Json(res))
  }
}
