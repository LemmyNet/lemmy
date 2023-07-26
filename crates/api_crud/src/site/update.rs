use crate::{
  site::{application_question_check, site_default_post_listing_type_check},
  PerformCrud,
};
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  site::{EditSite, SiteResponse},
  utils::{
    is_admin,
    local_site_rate_limit_to_rate_limit_config,
    local_user_view_from_jwt,
    sanitize_html_opt,
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
    tagline::Tagline,
  },
  traits::Crud,
  utils::{diesel_option_overwrite, diesel_option_overwrite_to_url, naive_now},
  RegistrationMode,
};
use lemmy_db_views::structs::SiteView;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType, LemmyResult},
  utils::{
    slurs::check_slurs_opt,
    validation::{
      build_and_check_regex,
      check_site_visibility_valid,
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
    let site_view = SiteView::read_local(&mut context.pool()).await?;
    let local_site = site_view.local_site;
    let site = site_view.site;

    // Make sure user is an admin; other types of users should not update site data...
    is_admin(&local_user_view)?;

    validate_update_payload(&local_site, data)?;

    if let Some(discussion_languages) = data.discussion_languages.clone() {
      SiteLanguage::update(&mut context.pool(), discussion_languages.clone(), &site).await?;
    }

    let name = sanitize_html_opt(&data.name);
    let sidebar = sanitize_html_opt(&data.sidebar);
    let description = sanitize_html_opt(&data.description);

    let site_form = SiteUpdateForm::builder()
      .name(name)
      .sidebar(diesel_option_overwrite(sidebar))
      .description(diesel_option_overwrite(description))
      .icon(diesel_option_overwrite_to_url(&data.icon)?)
      .banner(diesel_option_overwrite_to_url(&data.banner)?)
      .updated(Some(Some(naive_now())))
      .build();

    Site::update(&mut context.pool(), site.id, &site_form)
      .await
      // Ignore errors for all these, so as to not throw errors if no update occurs
      // Diesel will throw an error for empty update forms
      .ok();

    let application_question = sanitize_html_opt(&data.application_question);
    let default_theme = sanitize_html_opt(&data.default_theme);
    let legal_information = sanitize_html_opt(&data.legal_information);

    let local_site_form = LocalSiteUpdateForm::builder()
      .enable_downvotes(data.enable_downvotes)
      .registration_mode(data.registration_mode)
      .enable_nsfw(data.enable_nsfw)
      .community_creation_admin_only(data.community_creation_admin_only)
      .require_email_verification(data.require_email_verification)
      .application_question(diesel_option_overwrite(application_question))
      .private_instance(data.private_instance)
      .default_theme(default_theme)
      .default_post_listing_type(data.default_post_listing_type)
      .legal_information(diesel_option_overwrite(legal_information))
      .application_email_admins(data.application_email_admins)
      .hide_modlog_mod_names(data.hide_modlog_mod_names)
      .updated(Some(Some(naive_now())))
      .slur_filter_regex(diesel_option_overwrite(data.slur_filter_regex.clone()))
      .actor_name_max_length(data.actor_name_max_length)
      .federation_enabled(data.federation_enabled)
      .captcha_enabled(data.captcha_enabled)
      .captcha_difficulty(data.captcha_difficulty.clone())
      .reports_email_admins(data.reports_email_admins)
      .build();

    let update_local_site = LocalSite::update(&mut context.pool(), &local_site_form)
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

    LocalSiteRateLimit::update(&mut context.pool(), &local_site_rate_limit_form)
      .await
      .ok();

    // Replace the blocked and allowed instances
    let allowed = data.allowed_instances.clone();
    FederationAllowList::replace(&mut context.pool(), allowed).await?;
    let blocked = data.blocked_instances.clone();
    FederationBlockList::replace(&mut context.pool(), blocked).await?;

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
      LocalUser::set_all_users_registration_applications_accepted(&mut context.pool())
        .await
        .with_lemmy_type(LemmyErrorType::CouldntSetAllRegistrationsAccepted)?;
    }

    let new_require_email_verification = update_local_site
      .as_ref()
      .map(|ols| ols.require_email_verification)
      .unwrap_or(false);
    if !local_site.require_email_verification && new_require_email_verification {
      LocalUser::set_all_users_email_verified(&mut context.pool())
        .await
        .with_lemmy_type(LemmyErrorType::CouldntSetAllEmailVerified)?;
    }

    let new_taglines = data.taglines.clone();
    let taglines = Tagline::replace(&mut context.pool(), local_site.id, new_taglines).await?;

    let site_view = SiteView::read_local(&mut context.pool()).await?;

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

fn validate_update_payload(local_site: &LocalSite, edit_site: &EditSite) -> LemmyResult<()> {
  // Check that the slur regex compiles, and return the regex if valid...
  // Prioritize using new slur regex from the request; if not provided, use the existing regex.
  let slur_regex = build_and_check_regex(
    &edit_site
      .slur_filter_regex
      .as_deref()
      .or(local_site.slur_filter_regex.as_deref()),
  )?;

  if let Some(name) = &edit_site.name {
    // The name doesn't need to be updated, but if provided it cannot be blanked out...
    site_name_length_check(name)?;
    check_slurs_opt(&edit_site.name, &slur_regex)?;
  }

  if let Some(desc) = &edit_site.description {
    site_description_length_check(desc)?;
    check_slurs_opt(&edit_site.description, &slur_regex)?;
  }

  site_default_post_listing_type_check(&edit_site.default_post_listing_type)?;

  check_site_visibility_valid(
    local_site.private_instance,
    local_site.federation_enabled,
    &edit_site.private_instance,
    &edit_site.federation_enabled,
  )?;

  // Ensure that the sidebar has fewer than the max num characters...
  is_valid_body_field(&edit_site.sidebar, false)?;

  application_question_check(
    &local_site.application_question,
    &edit_site.application_question,
    edit_site
      .registration_mode
      .unwrap_or(local_site.registration_mode),
  )
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::site::update::validate_update_payload;
  use lemmy_api_common::site::EditSite;
  use lemmy_db_schema::{source::local_site::LocalSite, ListingType, RegistrationMode};
  use lemmy_utils::error::LemmyErrorType;

  #[test]
  fn test_validate_invalid_update_payload() {
    let invalid_payloads = [
      (
        "EditSite name matches LocalSite slur filter",
        LemmyErrorType::Slurs,
        &generate_local_site(
          Some(String::from("(foo|bar)")),
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_edit_site(
          Some(String::from("foo site_name")),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          None::<String>,
          None::<bool>,
          None::<bool>,
          None::<String>,
          None::<RegistrationMode>,
        ),
      ),
      (
        "EditSite name matches new slur filter",
        LemmyErrorType::Slurs,
        &generate_local_site(
          Some(String::from("(foo|bar)")),
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_edit_site(
          Some(String::from("zeta site_name")),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          Some(String::from("(zeta|alpha)")),
          None::<bool>,
          None::<bool>,
          None::<String>,
          None::<RegistrationMode>,
        ),
      ),
      (
        "EditSite listing type is Subscribed, which is invalid",
        LemmyErrorType::InvalidDefaultPostListingType,
        &generate_local_site(
          None::<String>,
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_edit_site(
          Some(String::from("site_name")),
          None::<String>,
          None::<String>,
          Some(ListingType::Subscribed),
          None::<String>,
          None::<bool>,
          None::<bool>,
          None::<String>,
          None::<RegistrationMode>,
        ),
      ),
      (
        "EditSite is both private and federated",
        LemmyErrorType::CantEnablePrivateInstanceAndFederationTogether,
        &generate_local_site(
          None::<String>,
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_edit_site(
          Some(String::from("site_name")),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          None::<String>,
          Some(true),
          Some(true),
          None::<String>,
          None::<RegistrationMode>,
        ),
      ),
      (
        "LocalSite is private, but EditSite also makes it federated",
        LemmyErrorType::CantEnablePrivateInstanceAndFederationTogether,
        &generate_local_site(
          None::<String>,
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_edit_site(
          Some(String::from("site_name")),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          None::<String>,
          None::<bool>,
          Some(true),
          None::<String>,
          None::<RegistrationMode>,
        ),
      ),
      (
        "EditSite requires application, but neither it nor LocalSite has an application question",
        LemmyErrorType::ApplicationQuestionRequired,
        &generate_local_site(
          None::<String>,
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_edit_site(
          Some(String::from("site_name")),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          None::<String>,
          None::<bool>,
          None::<bool>,
          None::<String>,
          Some(RegistrationMode::RequireApplication),
        ),
      ),
    ];

    invalid_payloads.iter().enumerate().for_each(
      |(
         idx,
         &(reason, ref expected_err, local_site, edit_site),
       )| {
        match validate_update_payload(local_site, edit_site) {
          Ok(_) => {
            panic!(
              "Got Ok, but validation should have failed with error: {} for reason: {}. invalid_payloads.nth({})",
              expected_err, reason, idx
            )
          }
          Err(error) => {
            assert!(
              error.error_type.eq(&expected_err.clone()),
              "Got Err {:?}, but should have failed with message: {} for reason: {}. invalid_payloads.nth({})",
              error.error_type,
              expected_err,
              reason,
              idx
            )
          }
        }
      },
    );
  }

  #[test]
  fn test_validate_valid_update_payload() {
    let valid_payloads = [
      (
        "No changes between LocalSite and EditSite",
        &generate_local_site(
          None::<String>,
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_edit_site(
          None::<String>,
          None::<String>,
          None::<String>,
          None::<ListingType>,
          None::<String>,
          None::<bool>,
          None::<bool>,
          None::<String>,
          None::<RegistrationMode>,
        ),
      ),
      (
        "EditSite allows clearing and changing values",
        &generate_local_site(
          None::<String>,
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_edit_site(
          Some(String::from("site_name")),
          Some(String::new()),
          Some(String::new()),
          Some(ListingType::All),
          Some(String::new()),
          Some(false),
          Some(true),
          Some(String::new()),
          Some(RegistrationMode::Open),
        ),
      ),
      (
        "EditSite name passes slur filter regex",
        &generate_local_site(
          Some(String::from("(foo|bar)")),
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_edit_site(
          Some(String::from("foo site_name")),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          Some(String::new()),
          None::<bool>,
          None::<bool>,
          None::<String>,
          None::<RegistrationMode>,
        ),
      ),
      (
        "LocalSite has application question and EditSite now requires applications,",
        &generate_local_site(
          None::<String>,
          true,
          false,
          Some(String::from("question")),
          RegistrationMode::Open,
        ),
        &generate_edit_site(
          Some(String::from("site_name")),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          None::<String>,
          None::<bool>,
          None::<bool>,
          None::<String>,
          Some(RegistrationMode::RequireApplication),
        ),
      ),
    ];

    valid_payloads
      .iter()
      .enumerate()
      .for_each(|(idx, &(reason, local_site, edit_site))| {
        assert!(
          validate_update_payload(local_site, edit_site).is_ok(),
          "Got Err, but should have got Ok for reason: {}. valid_payloads.nth({})",
          reason,
          idx
        );
      })
  }

  fn generate_local_site(
    site_slur_filter_regex: Option<String>,
    site_is_private: bool,
    site_is_federated: bool,
    site_application_question: Option<String>,
    site_registration_mode: RegistrationMode,
  ) -> LocalSite {
    LocalSite {
      id: Default::default(),
      site_id: Default::default(),
      site_setup: true,
      enable_downvotes: false,
      enable_nsfw: false,
      community_creation_admin_only: false,
      require_email_verification: false,
      application_question: site_application_question,
      private_instance: site_is_private,
      default_theme: String::new(),
      default_post_listing_type: ListingType::All,
      legal_information: None,
      hide_modlog_mod_names: false,
      application_email_admins: false,
      slur_filter_regex: site_slur_filter_regex,
      actor_name_max_length: 0,
      federation_enabled: site_is_federated,
      captcha_enabled: false,
      captcha_difficulty: String::new(),
      published: Default::default(),
      updated: None,
      registration_mode: site_registration_mode,
      reports_email_admins: false,
    }
  }

  // Allow the test helper function to have too many arguments.
  // It's either this or generate the entire struct each time for testing.
  #[allow(clippy::too_many_arguments)]
  fn generate_edit_site(
    site_name: Option<String>,
    site_description: Option<String>,
    site_sidebar: Option<String>,
    site_listing_type: Option<ListingType>,
    site_slur_filter_regex: Option<String>,
    site_is_private: Option<bool>,
    site_is_federated: Option<bool>,
    site_application_question: Option<String>,
    site_registration_mode: Option<RegistrationMode>,
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
      application_question: site_application_question,
      private_instance: site_is_private,
      default_theme: None,
      default_post_listing_type: site_listing_type,
      legal_information: None,
      application_email_admins: None,
      hide_modlog_mod_names: None,
      discussion_languages: None,
      slur_filter_regex: site_slur_filter_regex,
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
      federation_enabled: site_is_federated,
      federation_debug: None,
      captcha_enabled: None,
      captcha_difficulty: None,
      allowed_instances: None,
      blocked_instances: None,
      taglines: None,
      registration_mode: site_registration_mode,
      reports_email_admins: None,
      auth: Default::default(),
    }
  }
}
