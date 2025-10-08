use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{check_local_user_valid, get_multi_community, slur_regex},
};
use lemmy_db_schema::{
  source::multi_community::{MultiCommunity, MultiCommunityInsertForm},
  traits::{ApubActor, Crud},
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
  check_local_user_valid(&local_user_view)?;
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  is_valid_display_name(&data.name)?;

  let slur_regex = slur_regex(&context).await?;
  check_slurs(&data.name, &slur_regex)?;
  let ap_id = MultiCommunity::generate_local_actor_url(&data.name, context.settings())?;
  let following_url = Url::parse(&format!("{}/following", ap_id))?;

  let form = MultiCommunityInsertForm {
    title: data.title.clone(),
    description: data.description.clone(),
    ap_id: Some(ap_id),
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
  get_multi_community(multi.id, &context).await
}
