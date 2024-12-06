use crate::{fetcher::resolve_actor_identifier, objects::person::ApubPerson};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{GetPersonDetails, GetPersonDetailsResponse},
  utils::{check_private_instance, read_site_for_actor},
};
use lemmy_db_schema::source::person::Person;
use lemmy_db_views::{
  profile_combined_view::ProfileCombinedQuery,
  structs::{LocalUserView, SiteView},
};
use lemmy_db_views_actor::structs::{CommunityModeratorView, PersonView};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

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
          .await?
          .id
      } else {
        Err(LemmyErrorType::NotFound)?
      }
    }
  };

  // You don't need to return settings for the user, since this comes back with GetSite
  // `my_user`
  let person_view = PersonView::read(&mut context.pool(), person_details_id).await?;

  // parse pagination token
  let page_after = if let Some(pa) = &data.page_cursor {
    Some(pa.read(&mut context.pool()).await?)
  } else {
    None
  };
  let page_back = data.page_back;
  let saved_only = data.saved_only;
  let community_id = data.community_id;

  // If its saved only, then ignore the person details id,
  // and use your local user's id
  let creator_id = if !saved_only.unwrap_or_default() {
    Some(person_details_id)
  } else {
    local_user_view.as_ref().map(|u| u.local_user.person_id)
  };

  let content = if let Some(creator_id) = creator_id {
    ProfileCombinedQuery {
      creator_id,
      community_id,
      saved_only,
      page_after,
      page_back,
    }
    .list(&mut context.pool(), &local_user_view)
    .await?
  } else {
    // if the creator is missing (saved_only, and no local_user), then return empty content
    Vec::new()
  };

  let moderates = CommunityModeratorView::for_person(
    &mut context.pool(),
    person_details_id,
    local_user_view.map(|l| l.local_user).as_ref(),
  )
  .await?;

  let site = read_site_for_actor(person_view.person.actor_id.clone(), &context).await?;

  Ok(Json(GetPersonDetailsResponse {
    person_view,
    site,
    moderates,
    content,
  }))
}
