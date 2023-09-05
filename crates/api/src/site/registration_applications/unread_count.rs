use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
    context::LemmyContext,
    site::{GetUnreadRegistrationApplicationCount, GetUnreadRegistrationApplicationCountResponse},
    utils::{is_admin, local_user_view_from_jwt},
};
use lemmy_db_schema::source::local_site::LocalSite;
use lemmy_db_views::structs::RegistrationApplicationView;
use lemmy_utils::error::LemmyError;

pub async fn get_unread_registration_application_count(
    data: Query<GetUnreadRegistrationApplicationCount>,
    context: Data<LemmyContext>,
) -> Result<Json<GetUnreadRegistrationApplicationCountResponse>, LemmyError> {
    let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;
    let local_site = LocalSite::read(&mut context.pool()).await?;

    // Only let admins do this
    is_admin(&local_user_view)?;

    let verified_email_only = local_site.require_email_verification;

    let registration_applications =
        RegistrationApplicationView::get_unread_count(&mut context.pool(), verified_email_only)
            .await?;

    Ok(Json(GetUnreadRegistrationApplicationCountResponse {
        registration_applications,
    }))
}
