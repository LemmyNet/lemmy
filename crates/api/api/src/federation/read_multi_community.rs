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
  let id = match (data.id, &data.name) {
    (Some(id), _) => id,
    (_, Some(name)) => {
      resolve_ap_identifier::<ApubMultiCommunity, MultiCommunity>(
        name,
        &context,
        &local_user_view,
        true,
      )
      .await?
      .id
    }
    _ => Err(LemmyErrorType::NoIdGiven)?,
  };
  get_multi_community(id, &context, local_user_view).await
}
