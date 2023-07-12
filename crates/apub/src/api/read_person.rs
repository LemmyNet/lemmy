use crate::{fetcher::resolve_actor_identifier, objects::person::ApubPerson};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{GetPersonDetails, GetPersonDetailsResponse},
  utils::{check_private_instance, is_admin, local_user_view_from_jwt_opt},
};
use lemmy_db_schema::{
  source::{local_site::LocalSite, person::Person},
  utils::post_to_comment_sort_type,
};
use lemmy_db_views::{comment_view::CommentQuery, post_view::PostQuery};
use lemmy_db_views_actor::structs::{CommunityModeratorView, PersonView};
use lemmy_utils::error::{LemmyError, LemmyErrorExt2, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn read_person(
  data: Query<GetPersonDetails>,
  context: Data<LemmyContext>,
) -> Result<Json<GetPersonDetailsResponse>, LemmyError> {
  // Check to make sure a person name or an id is given
  if data.username.is_none() && data.person_id.is_none() {
    return Err(LemmyErrorType::NoIdGiven)?;
  }

  let local_user_view = local_user_view_from_jwt_opt(data.auth.as_ref(), &context).await;
  let local_site = LocalSite::read(&mut context.pool()).await?;
  let is_admin = local_user_view.as_ref().map(|luv| is_admin(luv).is_ok());

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
        return Err(LemmyErrorType::CouldntFindPerson)?;
      }
    }
  };

  // You don't need to return settings for the user, since this comes back with GetSite
  // `my_user`
  let person_view = PersonView::read(&mut context.pool(), person_details_id).await?;

  let sort = data.sort;
  let page = data.page;
  let limit = data.limit;
  let saved_only = data.saved_only;
  let community_id = data.community_id;
  let local_user = local_user_view.map(|l| l.local_user);
  let local_user_clone = local_user.clone();

  let posts = PostQuery::builder()
    .pool(&mut context.pool())
    .sort(sort)
    .saved_only(saved_only)
    .local_user(local_user.as_ref())
    .community_id(community_id)
    .is_mod_or_admin(is_admin)
    .page(page)
    .limit(limit)
    .creator_id(
      // If its saved only, you don't care what creator it was
      // Or, if its not saved, then you only want it for that specific creator
      if !saved_only.unwrap_or(false) {
        Some(person_details_id)
      } else {
        None
      },
    )
    .build()
    .list()
    .await?;

  let comments = CommentQuery::builder()
    .pool(&mut context.pool())
    .local_user(local_user_clone.as_ref())
    .sort(sort.map(post_to_comment_sort_type))
    .saved_only(saved_only)
    .show_deleted_and_removed(Some(false))
    .community_id(community_id)
    .page(page)
    .limit(limit)
    .creator_id(
      // If its saved only, you don't care what creator it was
      // Or, if its not saved, then you only want it for that specific creator
      if !saved_only.unwrap_or(false) {
        Some(person_details_id)
      } else {
        None
      },
    )
    .build()
    .list()
    .await?;

  let moderates =
    CommunityModeratorView::for_person(&mut context.pool(), person_details_id).await?;

  // Return the jwt
  Ok(Json(GetPersonDetailsResponse {
    person_view,
    moderates,
    comments,
    posts,
  }))
}
