use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
    context::LemmyContext,
    site::{ListRegistrationApplications, ListRegistrationApplicationsResponse},
    utils::{is_admin, local_user_view_from_jwt},
};
use lemmy_db_schema::source::local_site::LocalSite;
use lemmy_db_views::registration_application_view::RegistrationApplicationQuery;
use lemmy_utils::error::LemmyError;

/// Lists registration applications, filterable by undenied only.
pub async fn list_registration_applications(
    data: Query<ListRegistrationApplications>,
    context: Data<LemmyContext>,
) -> Result<Json<ListRegistrationApplicationsResponse>, LemmyError> {
    let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;
    let local_site = LocalSite::read(&mut context.pool()).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let unread_only = data.unread_only.unwrap_or_default();
    let verified_email_only = local_site.require_email_verification;

    let page = data.page;
    let limit = data.limit;
    let registration_applications = RegistrationApplicationQuery {
        unread_only,
        verified_email_only,
        page,
        limit,
    }
    .list(&mut context.pool())
    .await?;

    Ok(Json(ListRegistrationApplicationsResponse {
        registration_applications,
    }))
}
