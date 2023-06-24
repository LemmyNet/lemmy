use crate::{
  api::PerformApub,
  fetcher::search::{search_query_to_object_id, SearchableObjects},
};
use activitypub_federation::config::Data;
use diesel::NotFound;
use tracing::log::{log, logger, warn};
use lemmy_api_common::{
  context::LemmyContext,
  site::{ResolveObject, ResolveObjectResponse},
  utils::{check_private_instance, local_user_view_from_jwt},
};
use lemmy_db_schema::{newtypes::PersonId, source::local_site::LocalSite, utils::DbPool};
use lemmy_db_views::structs::{CommentView, PostView};
use lemmy_db_views_actor::structs::{CommunityView, PersonView};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait]
impl PerformApub for ResolveObject {
  type Response = ResolveObjectResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<ResolveObjectResponse, LemmyError> {
    let local_user_view = local_user_view_from_jwt(&self.auth, context).await?;
    let local_site = LocalSite::read(context.pool()).await?;
    let person_id = local_user_view.person.id;
    let check_private_instance_result = check_private_instance(&Some(local_user_view), &local_site);
    if let Err(e) = check_private_instance_result {
      warn!("Suppressed Error from check_private_instance: {}", e);
    }

    let object_id_result = search_query_to_object_id(&self.q, context)
      .await
      .map_err(|e| e.with_message("couldnt_find_object"));
    match object_id_result {
      Err(e) => warn!("Suppressed Error from search_query_to_object_id: {}", e),
      Ok(object_id) => {
        let convert_result = convert_response(object_id, person_id, context.pool())
            .await
            .map_err(|e| e.with_message("couldnt_find_object"));
        match convert_result {
          Err(e) => warn!("Suppressed Error from convert_response: {}", e),
          Ok(convert) => return Ok(convert)
        }
      }
    }

    Ok(ResolveObjectResponse::default())
  }
}

async fn convert_response(
  object: SearchableObjects,
  user_id: PersonId,
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
      res.community = Some(CommunityView::read(pool, c.id, Some(user_id), None).await?)
    }
    Post(p) => {
      removed_or_deleted = p.deleted || p.removed;
      res.post = Some(PostView::read(pool, p.id, Some(user_id), None).await?)
    }
    Comment(c) => {
      removed_or_deleted = c.deleted || c.removed;
      res.comment = Some(CommentView::read(pool, c.id, Some(user_id)).await?)
    }
  };
  // if the object was deleted from database, dont return it
  if removed_or_deleted {
    return Err(NotFound {}.into());
  }
  Ok(res)
}
