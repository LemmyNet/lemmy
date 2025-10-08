use crate::federation::resolve_ap_identifier;
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::get_multi_community};
use lemmy_apub_objects::objects::multi_community::ApubMultiCommunity;
use lemmy_db_schema::source::multi_community::MultiCommunity;
use lemmy_db_views_community::api::{GetMultiCommunity, GetMultiCommunityResponse};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn read_multi_community(
  data: Query<GetMultiCommunity>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetMultiCommunityResponse>> {
  if data.name.is_none() && data.id.is_none() {
    Err(LemmyErrorType::NoIdGiven)?
  }
  let id = match data.id {
    Some(id) => id,
    None => {
      let name = data.name.clone().expect("name was already checked");
      resolve_ap_identifier::<ApubMultiCommunity, MultiCommunity>(
        &name,
        &context,
        &local_user_view,
        true,
      )
      .await?
      .id
    }
  };
  get_multi_community(id, &context, local_user_view).await
}
