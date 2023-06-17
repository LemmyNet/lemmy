use crate::{site::check_application_question, PerformCrud};
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  site::{EditSite, SiteResponse},
  utils::{is_admin, local_site_rate_limit_to_rate_limit_config, local_user_view_from_jwt},
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
    tagline::Tagline,
  },
  traits::Crud,
  utils::{diesel_option_overwrite, diesel_option_overwrite_to_url, naive_now},
  ListingType,
  RegistrationMode,
};
use lemmy_db_views::structs::SiteView;
use lemmy_utils::{
  error::{LemmyError, LemmyResult},
  utils::{
    slurs::check_slurs_opt,
    validation::{
      build_and_check_regex,
      is_valid_body_field,
      site_description_length_check,
      site_name_length_check,
    },
  },
};

#[async_trait::async_trait(?Send)]
impl PerformCrud for EditSite {
  type Response = SiteResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<SiteResponse, LemmyError> {
    let data: &EditSite = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;
    let site_view = SiteView::read_local(context.pool()).await?;
    let local_site = site_view.local_site;
    let site = site_view.site;

    // Make sure user is an admin; other types of users should not update site data...
    is_admin(&local_user_view)?;

    validate_update_payload(
      local_site.slur_filter_regex,
      local_site.federation_enabled,
      local_site.private_instance,
      data,
    )?;

    let application_question = diesel_option_overwrite(&data.application_question);
    check_application_question(
      &application_question,
      data
        .registration_mode
        .unwrap_or(local_site.registration_mode),
    )?;

    if let Some(discussion_languages) = data.discussion_languages.clone() {
      SiteLanguage::update(context.pool(), discussion_languages.clone(), &site).await?;
    }

    let site_form = SiteUpdateForm::builder()
      .name(data.name.clone())
      .sidebar(diesel_option_overwrite(&data.sidebar))
      .description(diesel_option_overwrite(&data.description))
      .icon(diesel_option_overwrite_to_url(&data.icon)?)
      .banner(diesel_option_overwrite_to_url(&data.banner)?)
      .updated(Some(Some(naive_now())))
      .build();

    Site::update(context.pool(), site.id, &site_form)
      .await
      // Ignore errors for all these, so as to not throw errors if no update occurs
      // Diesel will throw an error for empty update forms
      .ok();

    let local_site_form = LocalSiteUpdateForm::builder()
      .enable_downvotes(data.enable_downvotes)
      .registration_mode(data.registration_mode)
      .enable_nsfw(data.enable_nsfw)
      .community_creation_admin_only(data.community_creation_admin_only)
      .require_email_verification(data.require_email_verification)
      .application_question(application_question)
      .private_instance(data.private_instance)
      .default_theme(data.default_theme.clone())
      .default_post_listing_type(data.default_post_listing_type)
      .legal_information(diesel_option_overwrite(&data.legal_information))
      .application_email_admins(data.application_email_admins)
      .hide_modlog_mod_names(data.hide_modlog_mod_names)
      .updated(Some(Some(naive_now())))
      .slur_filter_regex(diesel_option_overwrite(&data.slur_filter_regex))
      .actor_name_max_length(data.actor_name_max_length)
      .federation_enabled(data.federation_enabled)
      .federation_worker_count(data.federation_worker_count)
      .captcha_enabled(data.captcha_enabled)
      .captcha_difficulty(data.captcha_difficulty.clone())
      .reports_email_admins(data.reports_email_admins)
      .build();

    let update_local_site = LocalSite::update(context.pool(), &local_site_form)
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

    LocalSiteRateLimit::update(context.pool(), &local_site_rate_limit_form)
      .await
      .ok();

    // Replace the blocked and allowed instances
    let allowed = data.allowed_instances.clone();
    FederationAllowList::replace(context.pool(), allowed).await?;
    let blocked = data.blocked_instances.clone();
    FederationBlockList::replace(context.pool(), blocked).await?;

    // TODO can't think of a better way to do this.
    // If the server suddenly requires email verification, or required applications, no old users
    // will be able to log in. It really only wants this to be a requirement for NEW signups.
    // So if it was set from false, to true, you need to update all current users columns to be verified.

    let old_require_application =
      local_site.registration_mode == RegistrationMode::RequireApplication;
    let new_require_application = update_local_site
      .as_ref()
      .map(|ols| ols.registration_mode == RegistrationMode::RequireApplication)
      .unwrap_or(false);
    if !old_require_application && new_require_application {
      LocalUser::set_all_users_registration_applications_accepted(context.pool())
        .await
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_set_all_registrations_accepted"))?;
    }

    let new_require_email_verification = update_local_site
      .as_ref()
      .map(|ols| ols.require_email_verification)
      .unwrap_or(false);
    if !local_site.require_email_verification && new_require_email_verification {
      LocalUser::set_all_users_email_verified(context.pool())
        .await
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_set_all_email_verified"))?;
    }

    let new_taglines = data.taglines.clone();
    let taglines = Tagline::replace(context.pool(), local_site.id, new_taglines).await?;

    let site_view = SiteView::read_local(context.pool()).await?;

    let rate_limit_config =
      local_site_rate_limit_to_rate_limit_config(&site_view.local_site_rate_limit);
    context
      .settings_updated_channel()
      .send(rate_limit_config)
      .await?;

    let res = SiteResponse {
      site_view,
      taglines,
    };

    Ok(res)
  }
}

fn validate_update_payload(
  site_regex: Option<String>,
  federation_enabled: bool,
  private_instance: bool,
  edit_site: &EditSite,
) -> LemmyResult<()> {
  // Check that the slur regex compiles, and return the regex if valid...
  let slur_regex = build_and_check_regex(&site_regex.as_deref())?;

  if let Some(name) = &edit_site.name {
    // The name doesn't need to be updated, but if provided it cannot be blanked out...
    site_name_length_check(name)?;
    check_slurs_opt(&edit_site.name, &slur_regex)?;
  }

  if let Some(desc) = &edit_site.description {
    site_description_length_check(desc)?;
    check_slurs_opt(&edit_site.description, &slur_regex)?;
  }

  if let Some(listing_type) = &edit_site.default_post_listing_type {
    // Only allow all or local as default listing types...
    if listing_type != &ListingType::All && listing_type != &ListingType::Local {
      return Err(LemmyError::from_message(
        "invalid_default_post_listing_type",
      ));
    }
  }

  let enabled_private_instance_with_federation = edit_site.private_instance == Some(true)
    && edit_site.federation_enabled.unwrap_or(federation_enabled);
  let enabled_federation_with_private_instance = edit_site.federation_enabled == Some(true)
    && edit_site.private_instance.unwrap_or(private_instance);

  if enabled_private_instance_with_federation || enabled_federation_with_private_instance {
    return Err(LemmyError::from_message(
      "cant_enable_private_instance_and_federation_together",
    ));
  }

  // Ensure that the sidebar has fewer than the max num characters...
  is_valid_body_field(&edit_site.sidebar)
}

#[cfg(test)]
mod tests {
  use crate::site::update::validate_update_payload;
  use lemmy_api_common::site::EditSite;
  use lemmy_db_schema::ListingType;

  #[test]
  fn test_validate_create_invalid_payload() {
    fn create_payload(
      site_name: Option<String>,
      site_description: Option<String>,
      site_sidebar: Option<String>,
      site_listing_type: Option<ListingType>,
      is_private: Option<bool>,
      is_federation: Option<bool>,
    ) -> EditSite {
      EditSite {
        name: site_name,
        sidebar: site_sidebar,
        description: site_description,
        icon: None,
        banner: None,
        enable_downvotes: None,
        enable_nsfw: None,
        community_creation_admin_only: None,
        require_email_verification: None,
        application_question: None,
        private_instance: is_private,
        default_theme: None,
        default_post_listing_type: site_listing_type,
        legal_information: None,
        application_email_admins: None,
        hide_modlog_mod_names: None,
        discussion_languages: None,
        slur_filter_regex: None,
        actor_name_max_length: None,
        rate_limit_message: None,
        rate_limit_message_per_second: None,
        rate_limit_post: None,
        rate_limit_post_per_second: None,
        rate_limit_register: None,
        rate_limit_register_per_second: None,
        rate_limit_image: None,
        rate_limit_image_per_second: None,
        rate_limit_comment: None,
        rate_limit_comment_per_second: None,
        rate_limit_search: None,
        rate_limit_search_per_second: None,
        federation_enabled: is_federation,
        federation_debug: None,
        federation_worker_count: None,
        captcha_enabled: None,
        captcha_difficulty: None,
        allowed_instances: None,
        blocked_instances: None,
        taglines: None,
        registration_mode: None,
        reports_email_admins: None,
        auth: Default::default(),
      }
    }

    let invalid_payloads = [
      (
        &None,
        &create_payload(
          Some(String::from("site_name")),
          None::<String>,
          None::<String>,
          Some(ListingType::Subscribed),
          Some(true),
          Some(false),
        ),
        "invalid_default_post_listing_type",
      ),
      (
        &None,
        &create_payload(
          Some(String::from("site_name")),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          Some(true),
          Some(true),
        ),
        "cant_enable_private_instance_and_federation_together",
      ),
    ];

    let valid_payloads = [
      (
        &None::<String>,
        &create_payload(
          None::<String>,
          None::<String>,
          None::<String>,
          None::<ListingType>,
          Some(true),
          Some(false),
        ),
      ),
      (
        &Some(String::new()),
        &create_payload(
          Some(String::from("site_name")),
          Some(String::new()),
          Some(String::new()),
          Some(ListingType::All),
          Some(false),
          Some(true),
        ),
      ),
    ];

    invalid_payloads.iter().enumerate().for_each(
      |(idx, &(site_regex, edit_site, expected_err))| match validate_update_payload(
        site_regex.clone(),
        false,
        true,
        edit_site,
      ) {
        Ok(_) => {
          panic!(
            "Got Ok, but validation should have failed with error: {} for invalid_payloads.nth({})",
            expected_err, idx
          )
        }
        Err(error) => {
          assert!(
            error.message.eq(&Some(String::from(expected_err))),
            "Got Err {:?}, but should have failed with message: {} for invalid_payloads.nth({})",
            error.message,
            expected_err,
            idx
          )
        }
      },
    );

    valid_payloads
      .iter()
      .enumerate()
      .for_each(|(idx, &(site_regex, edit_site))| {
        let result = validate_update_payload(site_regex.clone(), true, false, edit_site);

        assert!(
          result.is_ok(),
          "Got Err, but should have got Ok for valid_payloads.nth({})",
          idx
        );
      })
  }
}
