use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
    context::LemmyContext,
    person::{BannedPersonsResponse, GetBannedPersons},
    utils::{is_admin, local_user_view_from_jwt},
};
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::error::LemmyError;

pub async fn list_banned_users(
    data: Query<GetBannedPersons>,
    context: Data<LemmyContext>,
) -> Result<Json<BannedPersonsResponse>, LemmyError> {
    let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let banned = PersonView::banned(&mut context.pool()).await?;

    Ok(Json(BannedPersonsResponse { banned }))
}
