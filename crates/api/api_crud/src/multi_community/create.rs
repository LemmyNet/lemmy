use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{check_local_user_valid, slur_regex},
};
use lemmy_db_schema::{
  source::multi_community::{MultiCommunity, MultiCommunityFollowForm, MultiCommunityInsertForm},
  traits::ApubActor,
};
use lemmy_db_schema_file::enums::CommunityFollowerState;
use lemmy_db_views_community::{
  MultiCommunityView,
  api::{CreateMultiCommunity, MultiCommunityResponse},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::{
  error::LemmyResult,
  utils::{slurs::check_slurs, validation::is_valid_display_name},
};
use url::Url;

pub async fn create_multi_community(
  Json(data): Json<CreateMultiCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<MultiCommunityResponse>> {
  check_local_user_valid(&local_user_view)?;
  let site_view = SiteView::read_local(&mut context.pool()).await?;

  let my_person_id = local_user_view.person.id;
  is_valid_display_name(&data.name)?;

  let slur_regex = slur_regex(&context).await?;
  check_slurs(&data.name, &slur_regex)?;
  let ap_id = MultiCommunity::generate_local_actor_url(&data.name, context.settings())?;
  let following_url = Url::parse(&format!("{}/following", ap_id))?;

  let form = MultiCommunityInsertForm {
    title: data.title.clone(),
    summary: data.summary.clone(),
    ap_id: Some(ap_id),
    private_key: site_view.site.private_key,
    inbox_url: Some(site_view.site.inbox_url),
    following_url: Some(following_url.into()),
    ..MultiCommunityInsertForm::new(
      my_person_id,
      local_user_view.person.instance_id,
      data.name.clone(),
      site_view.site.public_key,
    )
  };

  let multi = MultiCommunity::create(&mut context.pool(), &form).await?;

  // You follow your own community
  let follow_form = MultiCommunityFollowForm {
    multi_community_id: multi.id,
    person_id: my_person_id,
    follow_state: CommunityFollowerState::Accepted,
  };
  MultiCommunity::follow(&mut context.pool(), &follow_form).await?;

  let multi_community_view =
    MultiCommunityView::read(&mut context.pool(), multi.id, Some(my_person_id)).await?;

  Ok(Json(MultiCommunityResponse {
    multi_community_view,
  }))
}
