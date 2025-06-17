use crate::multi_community::get_multi;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{context::LemmyContext, utils::slur_regex};
use lemmy_db_schema::{
  source::multi_community::{MultiCommunity, MultiCommunityInsertForm},
  traits::Crud,
};
use lemmy_db_views_community::api::{CreateMultiCommunity, GetMultiCommunityResponse};
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
) -> LemmyResult<Json<GetMultiCommunityResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  is_valid_display_name(&data.name, site_view.local_site.actor_name_max_length)?;

  let slur_regex = slur_regex(&context).await?;
  check_slurs(&data.name, &slur_regex)?;
  let ap_id = Url::parse(&format!(
    "{}/m/{}",
    context.settings().get_protocol_and_hostname(),
    &data.name
  ))?;
  let following_url = Url::parse(&format!("{}/following", ap_id))?;

  let form = MultiCommunityInsertForm {
    title: data.title.clone(),
    description: data.description.clone(),
    ap_id: Some(ap_id.into()),
    private_key: site_view.site.private_key,
    inbox_url: Some(site_view.site.inbox_url),
    following_url: Some(following_url.into()),
    ..MultiCommunityInsertForm::new(
      local_user_view.person.id,
      local_user_view.person.instance_id,
      data.name.clone(),
      site_view.site.public_key,
    )
  };
  let multi = MultiCommunity::create(&mut context.pool(), &form).await?;
  get_multi(multi.id, context).await
}
