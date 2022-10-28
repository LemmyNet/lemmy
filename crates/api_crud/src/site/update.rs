use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  site::{EditSite, SiteResponse},
  utils::{
    blocking,
    get_local_user_view_from_jwt,
    is_admin,
    local_site_to_slur_regex,
    site_description_length_check,
  },
};
use lemmy_db_schema::{
  source::{
    actor_language::SiteLanguage,
    federation_allowlist::FederationAllowList,
    federation_blocklist::FederationBlockList,
    local_site::{LocalSite, LocalSiteUpdateForm},
    local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitUpdateForm},
    local_user::LocalUser,
    site::{Site, SiteUpdateForm},
  },
  traits::Crud,
  utils::{diesel_option_overwrite, diesel_option_overwrite_to_url, naive_now},
  ListingType,
};
use lemmy_db_views::structs::SiteView;
use lemmy_utils::{
  error::LemmyError,
  utils::{check_application_question, check_slurs_opt},
  ConnectionId,
};
use lemmy_websocket::{messages::SendAllMessage, LemmyContext, UserOperationCrud};
use std::str::FromStr;

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
    let local_site = blocking(context.pool(), LocalSite::read).await??;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let slur_regex = local_site_to_slur_regex(&local_site);

    check_slurs_opt(&data.name, &slur_regex)?;
    check_slurs_opt(&data.description, &slur_regex)?;

    if let Some(desc) = &data.description {
      site_description_length_check(desc)?;
    }

    let application_question = diesel_option_overwrite(&data.application_question);
    check_application_question(&application_question, &data.require_application)?;

    if let Some(default_post_listing_type) = &data.default_post_listing_type {
      // only allow all or local as default listing types
      let val = ListingType::from_str(default_post_listing_type);
      if val != Ok(ListingType::All) && val != Ok(ListingType::Local) {
        return Err(LemmyError::from_message(
          "invalid_default_post_listing_type",
        ));
      }
    }

    let site_id = local_site.site_id;
    if let Some(discussion_languages) = data.discussion_languages.clone() {
      blocking(context.pool(), move |conn| {
        let site = Site::read(conn, site_id)?;
        SiteLanguage::update(conn, discussion_languages, &site)
      })
      .await??;
    }

    let name = data.name.to_owned();
    let site_form = SiteUpdateForm::builder()
      .name(name)
      .sidebar(diesel_option_overwrite(&data.sidebar))
      .description(diesel_option_overwrite(&data.description))
      .icon(diesel_option_overwrite_to_url(&data.icon)?)
      .banner(diesel_option_overwrite_to_url(&data.banner)?)
      .updated(Some(Some(naive_now())))
      .build();

    blocking(context.pool(), move |conn| {
      Site::update(conn, local_site.site_id, &site_form)
    })
    .await
    // Ignore errors for all these, so as to not throw errors if no update occurs
    // Diesel will throw an error for empty update forms
    .ok();

    let local_site_form = LocalSiteUpdateForm::builder()
      .enable_downvotes(data.enable_downvotes)
      .open_registration(data.open_registration)
      .enable_nsfw(data.enable_nsfw)
      .community_creation_admin_only(data.community_creation_admin_only)
      .require_email_verification(data.require_email_verification)
      .require_application(data.require_application)
      .application_question(application_question)
      .private_instance(data.private_instance)
      .default_theme(data.default_theme.clone())
      .default_post_listing_type(data.default_post_listing_type.clone())
      .legal_information(diesel_option_overwrite(&data.legal_information))
      .application_email_admins(data.application_email_admins)
      .hide_modlog_mod_names(data.hide_modlog_mod_names)
      .updated(Some(Some(naive_now())))
      .slur_filter_regex(diesel_option_overwrite(&data.slur_filter_regex))
      .actor_name_max_length(data.actor_name_max_length)
      .federation_enabled(data.federation_enabled)
      .federation_debug(data.federation_debug)
      .federation_strict_allowlist(data.federation_strict_allowlist)
      .federation_http_fetch_retry_limit(data.federation_http_fetch_retry_limit)
      .federation_worker_count(data.federation_worker_count)
      .captcha_enabled(data.captcha_enabled)
      .captcha_difficulty(data.captcha_difficulty.to_owned())
      .build();

    let update_local_site = blocking(context.pool(), move |conn| {
      LocalSite::update(conn, &local_site_form)
    })
    .await
    .ok();

    let local_site_rate_limit_form = LocalSiteRateLimitUpdateForm::builder()
      .message(data.rate_limit_message)
      .message_per_second(data.rate_limit_message_per_second)
      .post(data.rate_limit_post)
      .post_per_second(data.rate_limit_post_per_second)
      .register(data.rate_limit_register)
      .register_per_second(data.rate_limit_register_per_second)
      .image(data.rate_limit_image)
      .image_per_second(data.rate_limit_image_per_second)
      .comment(data.rate_limit_comment)
      .comment_per_second(data.rate_limit_comment_per_second)
      .search(data.rate_limit_search)
      .search_per_second(data.rate_limit_search_per_second)
      .build();

    blocking(context.pool(), move |conn| {
      LocalSiteRateLimit::update(conn, &local_site_rate_limit_form)
    })
    .await
    .ok();

    // Replace the blocked and allowed instances
    let allowed = data.allowed_instances.to_owned();
    blocking(context.pool(), move |conn| {
      FederationAllowList::replace(conn, allowed)
    })
    .await??;
    let blocked = data.blocked_instances.to_owned();
    blocking(context.pool(), move |conn| {
      FederationBlockList::replace(conn, blocked)
    })
    .await??;

    // TODO can't think of a better way to do this.
    // If the server suddenly requires email verification, or required applications, no old users
    // will be able to log in. It really only wants this to be a requirement for NEW signups.
    // So if it was set from false, to true, you need to update all current users columns to be verified.

    let new_require_application = update_local_site
      .as_ref()
      .map(|ols| {
        ols
          .as_ref()
          .map(|ls| ls.require_application)
          .unwrap_or(false)
      })
      .unwrap_or(false);
    if !local_site.require_application && new_require_application {
      blocking(context.pool(), move |conn| {
        LocalUser::set_all_users_registration_applications_accepted(conn)
      })
      .await?
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_set_all_registrations_accepted"))?;
    }

    let new_require_email_verification = update_local_site
      .as_ref()
      .map(|ols| {
        ols
          .as_ref()
          .map(|ls| ls.require_email_verification)
          .unwrap_or(false)
      })
      .unwrap_or(false);
    if !local_site.require_email_verification && new_require_email_verification {
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
