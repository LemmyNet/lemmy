use crate::federation::fetcher::resolve_person_identifier;
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{check_private_instance, is_admin, read_site_for_actor},
};
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::{
  PersonView,
  api::{GetPersonDetails, GetPersonDetailsResponse},
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn read_person(
  data: Query<GetPersonDetails>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetPersonDetailsResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = site_view.local_site;
  let local_instance_id = site_view.site.instance_id;
  let my_person_id = local_user_view.as_ref().map(|l| l.person.id);

  check_private_instance(&local_user_view, &local_site)?;

  let person_details_id =
    resolve_person_identifier(data.person_id, &data.username, &context, &local_user_view).await?;

  // You don't need to return settings for the user, since this comes back with GetSite
  // `my_user`
  let is_admin = local_user_view
    .as_ref()
    .map(|l| is_admin(l).is_ok())
    .unwrap_or_default();

  let person_view = PersonView::read(
    &mut context.pool(),
    person_details_id,
    my_person_id,
    local_instance_id,
    is_admin,
  )
  .await?;
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
