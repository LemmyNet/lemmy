use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{community::CreateMultiCommunity, context::LemmyContext};
use lemmy_db_schema::source::multi_community::{MultiCommunity, MultiCommunityInsertForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;
use url::Url;

pub async fn create_multi_community(
  data: Json<CreateMultiCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<MultiCommunity>> {
  // TODO: length check
  let form = MultiCommunityInsertForm {
    owner_id: local_user_view.person.id,
    name: data.name.clone(),
    ap_id: Url::parse(&format!(
      "{}/m/{}",
      &local_user_view.person.ap_id, &data.name
    ))?
    .into(),
  };
  let res = MultiCommunity::create(&mut context.pool(), &form).await?;
  Ok(Json(res))
}
