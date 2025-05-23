use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{community::CreateMultiCommunity, context::LemmyContext, utils::slur_regex};
use lemmy_db_schema::{
  source::multi_community::{MultiCommunity, MultiCommunityInsertForm},
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::LemmyResult,
  utils::{slurs::check_slurs, validation::is_valid_display_name},
};
use url::Url;

pub async fn create_multi_community(
  data: Json<CreateMultiCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<MultiCommunity>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  is_valid_display_name(&data.name, local_site.actor_name_max_length)?;

  let slur_regex = slur_regex(&context).await?;
  check_slurs(&data.name, &slur_regex)?;

  let form = MultiCommunityInsertForm {
    creator_id: local_user_view.person.id,
    name: data.name.clone(),
    title: data.title.clone(),
    local: Some(true),
    description: data.description.clone(),
    ap_id: Url::parse(&format!(
      "{}/m/{}",
      context.settings().get_protocol_and_hostname(),
      &data.name
    ))?
    .into(),
  };
  let res = MultiCommunity::create(&mut context.pool(), &form).await?;
  Ok(Json(res))
}
