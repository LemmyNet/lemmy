use super::resolve_person_id_from_id_or_username;
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  person::{GetPersonDetails, GetPersonDetailsResponse},
  utils::{check_private_instance, is_admin, read_site_for_actor},
};
use lemmy_db_views::structs::{CommunityModeratorView, LocalUserView, PersonView, SiteView};
use lemmy_utils::error::LemmyResult;

pub async fn read_person(
  data: Query<GetPersonDetails>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetPersonDetailsResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site.local_site)?;

  let person_details_id = resolve_person_id_from_id_or_username(
    &data.person_id,
    &data.username,
    &context,
    &local_user_view,
  )
  .await?;

  // You don't need to return settings for the user, since this comes back with GetSite
  // `my_user`
  let is_admin = local_user_view
    .as_ref()
    .map(|l| is_admin(l).is_ok())
    .unwrap_or_default();
  let person_view = PersonView::read(&mut context.pool(), person_details_id, is_admin).await?;
  let moderates = CommunityModeratorView::for_person(
    &mut context.pool(),
    person_details_id,
    local_user_view.map(|l| l.local_user).as_ref(),
  )
  .await?;

  let site = read_site_for_actor(person_view.person.ap_id.clone(), &context).await?;

  Ok(Json(GetPersonDetailsResponse {
    person_view,
    site,
    moderates,
  }))
}
