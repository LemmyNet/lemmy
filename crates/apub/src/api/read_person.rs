use crate::fetcher::resolve_ap_identifier;
use activitypub_federation::config::Data;
use actix_web::{web::Json, HttpRequest};
use lemmy_api_utils::{
  context::LemmyContext,
  request::parse_person_id_or_name_from_request,
  utils::{check_private_instance, is_admin, read_site_for_actor},
};
use lemmy_apub_objects::objects::person::ApubPerson;
use lemmy_db_schema::source::person::Person;
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::{
  api::{GetPersonDetailsResponse, PersonIdOrName},
  PersonView,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn read_person(
  req: HttpRequest,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetPersonDetailsResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = site_view.local_site;
  let local_instance_id = site_view.site.instance_id;
  let my_person_id = local_user_view.as_ref().map(|l| l.person.id);

  check_private_instance(&local_user_view, &local_site)?;

  let person_id_or_name = parse_person_id_or_name_from_request(&req)?;
  let person_details_id = match person_id_or_name {
    PersonIdOrName::Id(id) => id,
    PersonIdOrName::Name(username) => {
      resolve_ap_identifier::<ApubPerson, Person>(username, &context, &local_user_view, true)
        .await?
        .id
    }
  };

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
