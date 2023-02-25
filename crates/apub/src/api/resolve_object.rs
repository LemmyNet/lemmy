use crate::{
  api::PerformApub,
  fetcher::search::{search_query_to_object_id, SearchableObjects},
};
use actix_web::web::Data;
use diesel::NotFound;
use lemmy_api_common::{
  context::LemmyContext,
  site::{ResolveObject, ResolveObjectResponse},
  utils::{check_private_instance, get_local_user_view_from_jwt_opt},
};
use lemmy_db_schema::{newtypes::PersonId, source::local_site::LocalSite, utils::DbPool};
use lemmy_db_views::structs::{CommentView, PostView};
use lemmy_db_views_actor::structs::{CommunityView, PersonView};
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl PerformApub for ResolveObject {
  type Response = ResolveObjectResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<ResolveObjectResponse, LemmyError> {
    let local_user_view =
      get_local_user_view_from_jwt_opt(self.auth.as_ref(), context.pool(), context.secret())
        .await?;
    let local_site = LocalSite::read(context.pool()).await?;
    check_private_instance(&local_user_view, &local_site)?;

    // In release builds only allow for authenticated users to fetch remote objects
    let local_only = local_user_view.is_none() && cfg!(not(debug_assertions));
    let res = search_query_to_object_id(&self.q, local_only, context)
      .await
      .map_err(|e| e.with_message("couldnt_find_object"))?;
    convert_response(res, local_user_view.map(|l| l.person.id), context.pool())
      .await
      .map_err(|e| e.with_message("couldnt_find_object"))
  }
}

async fn convert_response(
  object: SearchableObjects,
  user_id: Option<PersonId>,
  pool: &DbPool,
) -> Result<ResolveObjectResponse, LemmyError> {
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
      res.community = Some(CommunityView::read(pool, c.id, user_id).await?)
    }
    Post(p) => {
      removed_or_deleted = p.deleted || p.removed;
      res.post = Some(PostView::read(pool, p.id, user_id).await?)
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
  Ok(res)
}
