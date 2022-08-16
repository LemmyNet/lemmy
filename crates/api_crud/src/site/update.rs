use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  site::{EditSite, SiteResponse},
  utils::{blocking, get_local_user_view_from_jwt, is_admin, site_description_length_check},
};
use lemmy_db_schema::{
  source::{
    local_user::LocalUser,
    site::{Site, SiteForm},
  },
  traits::Crud,
  utils::{diesel_option_overwrite, diesel_option_overwrite_to_url, naive_now},
  ListingType,
};
use lemmy_db_views::structs::SiteView;
use lemmy_utils::{error::LemmyError, utils::check_slurs_opt, ConnectionId};
use lemmy_websocket::{messages::SendAllMessage, LemmyContext, UserOperationCrud};
use std::{default::Default, str::FromStr};

#[async_trait::async_trait(?Send)]
impl PerformCrud for EditSite {
  type Response = SiteResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<SiteResponse, LemmyError> {
    let data: &EditSite = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let local_site = blocking(context.pool(), Site::read_local_site).await??;

    let sidebar = diesel_option_overwrite(&data.sidebar);
    let description = diesel_option_overwrite(&data.description);
    let application_question = diesel_option_overwrite(&data.application_question);
    let legal_information = diesel_option_overwrite(&data.legal_information);
    let icon = diesel_option_overwrite_to_url(&data.icon)?;
    let banner = diesel_option_overwrite_to_url(&data.banner)?;

    check_slurs_opt(&data.name, &context.settings().slur_regex())?;
    check_slurs_opt(&data.description, &context.settings().slur_regex())?;

    if let Some(Some(desc)) = &description {
      site_description_length_check(desc)?;
    }

    // Make sure if applications are required, that there is an application questionnaire
    if data.require_application.unwrap_or(false)
      && application_question.as_ref().unwrap_or(&None).is_none()
    {
      return Err(LemmyError::from_message("application_question_required"));
    }

    if let Some(default_post_listing_type) = &data.default_post_listing_type {
      // only allow all or local as default listing types
      let val = ListingType::from_str(default_post_listing_type);
      if val != Ok(ListingType::All) && val != Ok(ListingType::Local) {
        return Err(LemmyError::from_message(
          "invalid_default_post_listing_type",
        ));
      }
    }

    let site_form = SiteForm {
      name: data.name.to_owned().unwrap_or(local_site.name),
      sidebar,
      description,
      icon,
      banner,
      updated: Some(naive_now()),
      enable_downvotes: data.enable_downvotes,
      open_registration: data.open_registration,
      enable_nsfw: data.enable_nsfw,
      community_creation_admin_only: data.community_creation_admin_only,
      require_email_verification: data.require_email_verification,
      require_application: data.require_application,
      application_question,
      private_instance: data.private_instance,
      default_theme: data.default_theme.clone(),
      default_post_listing_type: data.default_post_listing_type.clone(),
      legal_information,
      hide_modlog_mod_names: data.hide_modlog_mod_names,
      ..SiteForm::default()
    };

    let update_site = blocking(context.pool(), move |conn| {
      Site::update(conn, local_site.id, &site_form)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_site"))?;

    // TODO can't think of a better way to do this.
    // If the server suddenly requires email verification, or required applications, no old users
    // will be able to log in. It really only wants this to be a requirement for NEW signups.
    // So if it was set from false, to true, you need to update all current users columns to be verified.

    if !local_site.require_application && update_site.require_application {
      blocking(context.pool(), move |conn| {
        LocalUser::set_all_users_registration_applications_accepted(conn)
      })
      .await?
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_set_all_registrations_accepted"))?;
    }

    if !local_site.require_email_verification && update_site.require_email_verification {
      blocking(context.pool(), move |conn| {
        LocalUser::set_all_users_email_verified(conn)
      })
      .await?
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_set_all_email_verified"))?;
    }

    let site_view = blocking(context.pool(), SiteView::read_local).await??;

    let res = SiteResponse { site_view };

    context.chat_server().do_send(SendAllMessage {
      op: UserOperationCrud::EditSite,
      response: res.clone(),
      websocket_id,
    });

    Ok(res)
  }
}
