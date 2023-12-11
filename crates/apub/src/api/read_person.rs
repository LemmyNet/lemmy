use crate::{fetcher::resolve_actor_identifier, objects::person::ApubPerson};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{GetPersonDetails, GetPersonDetailsResponse},
  utils::check_private_instance,
};
use lemmy_db_schema::{
  source::{local_site::LocalSite, person::Person},
  utils::post_to_comment_sort_type,
};
use lemmy_db_views::{comment_view::CommentQuery, post_view::PostQuery, structs::LocalUserView};
use lemmy_db_views_actor::structs::{CommunityModeratorView, PersonView};
use lemmy_utils::error::{LemmyError, LemmyErrorExt2, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn read_person(
  data: Query<GetPersonDetails>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> Result<Json<GetPersonDetailsResponse>, LemmyError> {
  // Check to make sure a person name or an id is given
  if data.username.is_none() && data.person_id.is_none() {
    Err(LemmyErrorType::NoIdGiven)?
  }

  let local_site = LocalSite::read(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site)?;

  let person_details_id = match data.person_id {
    Some(id) => id,
    None => {
      if let Some(username) = &data.username {
        resolve_actor_identifier::<ApubPerson, Person>(username, &context, &local_user_view, true)
          .await
          .with_lemmy_type(LemmyErrorType::CouldntFindPerson)?
          .id
      } else {
        Err(LemmyErrorType::CouldntFindPerson)?
      }
    }
  };

  // You don't need to return settings for the user, since this comes back with GetSite
  // `my_user`
  let person_view = PersonView::read(&mut context.pool(), person_details_id).await?;

  let sort = data.sort;
  let page = data.page;
  let limit = data.limit;
  let saved_only = data.saved_only.unwrap_or_default();
  let community_id = data.community_id;
  // If its saved only, you don't care what creator it was
  // Or, if its not saved, then you only want it for that specific creator
  let creator_id = if !saved_only {
    Some(person_details_id)
  } else {
    None
  };

  let posts = PostQuery {
    sort,
    saved_only,
    local_user: local_user_view.as_ref(),
    community_id,
    is_profile_view: true,
    page,
    limit,
    creator_id,
    ..Default::default()
  }
  .list(&mut context.pool())
  .await?;

  let comments = CommentQuery {
    local_user: local_user_view.as_ref(),
    sort: sort.map(post_to_comment_sort_type),
    saved_only,
    community_id,
    is_profile_view: true,
    page,
    limit,
    creator_id,
    ..Default::default()
  }
  .list(&mut context.pool())
  .await?;

  let moderates =
    CommunityModeratorView::for_person(context.inner_pool(), person_details_id).await?;

  // Return the jwt
  Ok(Json(GetPersonDetailsResponse {
    person_view,
    moderates,
    comments,
    posts,
  }))
}
