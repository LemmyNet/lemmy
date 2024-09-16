use crate::{fetcher::resolve_actor_identifier, objects::person::ApubPerson};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{GetPersonDetails, GetPersonDetailsResponse},
  utils::{check_private_instance, read_site_for_actor},
};
use lemmy_db_schema::{source::person::Person, utils::post_to_comment_sort_type};
use lemmy_db_views::{
  comment_view::CommentQuery,
  post_view::PostQuery,
  structs::{LocalUserView, SiteView},
};
use lemmy_db_views_actor::structs::{CommunityModeratorView, PersonView};
use lemmy_utils::error::{LemmyErrorExt2, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn read_person(
  data: Query<GetPersonDetails>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetPersonDetailsResponse>> {
  // Check to make sure a person name or an id is given
  if data.username.is_none() && data.person_id.is_none() {
    Err(LemmyErrorType::NoIdGiven)?
  }

  let local_site = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site.local_site)?;

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
  let person_view = PersonView::read(&mut context.pool(), person_details_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPerson)?;

  let sort = data.sort;
  let page = data.page;
  let limit = data.limit;
  let saved_only = data.saved_only;
  let community_id = data.community_id;
  // If its saved only, you don't care what creator it was
  // Or, if its not saved, then you only want it for that specific creator
  let creator_id = if !saved_only.unwrap_or_default() {
    Some(person_details_id)
  } else {
    None
  };

  let local_user = local_user_view.as_ref().map(|l| &l.local_user);

  let posts = PostQuery {
    sort,
    saved_only,
    local_user,
    community_id,
    page,
    limit,
    creator_id,
    ..Default::default()
  }
  .list(&local_site.site, &mut context.pool())
  .await?;

  let comments = CommentQuery {
    local_user,
    sort: sort.map(post_to_comment_sort_type),
    saved_only,
    community_id,
    page,
    limit,
    creator_id,
    ..Default::default()
  }
  .list(&mut context.pool())
  .await?;

  let moderates = CommunityModeratorView::for_person(
    &mut context.pool(),
    person_details_id,
    local_user_view.map(|l| l.local_user).as_ref(),
  )
  .await?;

  let site = read_site_for_actor(person_view.person.actor_id.clone(), &context).await?;

  // Return the jwt
  Ok(Json(GetPersonDetailsResponse {
    person_view,
    site,
    moderates,
    comments,
    posts,
  }))
}
