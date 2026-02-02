use super::not_zero;
use crate::site::{application_question_check, site_default_post_listing_type_check};
use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{
    get_url_blocklist,
    is_admin,
    local_site_rate_limit_to_rate_limit_config,
    process_markdown_opt,
    slur_regex,
  },
};
use lemmy_db_schema::source::{
  actor_language::SiteLanguage,
  local_site::{LocalSite, LocalSiteUpdateForm},
  local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitUpdateForm},
  local_site_url_blocklist::LocalSiteUrlBlocklist,
  local_user::LocalUser,
  site::{Site, SiteUpdateForm},
};
use lemmy_db_schema_file::enums::RegistrationMode;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::{
  SiteView,
  api::{EditSite, SiteResponse},
};
use lemmy_diesel_utils::{
  traits::Crud,
  utils::{diesel_opt_number_update, diesel_string_update},
};
use lemmy_utils::{
  error::LemmyResult,
  utils::{
    slurs::check_slurs_opt,
    validation::{
      build_and_check_regex,
      check_urls_are_valid,
      is_valid_body_field,
      site_name_length_check,
      summary_length_check,
    },
  },
};

pub async fn edit_site(
  Json(data): Json<EditSite>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SiteResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = site_view.local_site;
  let site = site_view.site;

  // Make sure user is an admin; other types of users should not update site data...
  is_admin(&local_user_view)?;

  validate_update_payload(&local_site, &data)?;

  if let Some(discussion_languages) = data.discussion_languages.clone() {
    SiteLanguage::update(&mut context.pool(), discussion_languages.clone(), &site).await?;
  }

  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  let sidebar = diesel_string_update(
    process_markdown_opt(&data.sidebar, &slur_regex, &url_blocklist, &context)
      .await?
      .as_deref(),
  );
  let default_post_time_range_seconds =
    diesel_opt_number_update(data.default_post_time_range_seconds);
  let default_items_per_page = data.default_items_per_page;

  let site_form = SiteUpdateForm {
    name: data.name.clone(),
    sidebar,
    summary: diesel_string_update(data.summary.as_deref()),
    content_warning: diesel_string_update(data.content_warning.as_deref()),
    updated_at: Some(Some(Utc::now())),
    ..Default::default()
  };

  Site::update(&mut context.pool(), site.id, &site_form)
    .await
    // Ignore errors for all these, so as to not throw errors if no update occurs
    // Diesel will throw an error for empty update forms
    .ok();

  let local_site_form = LocalSiteUpdateForm {
    registration_mode: data.registration_mode,
    community_creation_admin_only: data.community_creation_admin_only,
    require_email_verification: data.require_email_verification,
    application_question: diesel_string_update(data.application_question.as_deref()),
    private_instance: data.private_instance,
    default_theme: data.default_theme.clone(),
    default_post_listing_type: data.default_post_listing_type,
    default_post_sort_type: data.default_post_sort_type,
    default_post_time_range_seconds,
    default_items_per_page,
    default_comment_sort_type: data.default_comment_sort_type,
    legal_information: diesel_string_update(data.legal_information.as_deref()),
    application_email_admins: data.application_email_admins,
    updated_at: Some(Some(Utc::now())),
    slur_filter_regex: diesel_string_update(data.slur_filter_regex.as_deref()),
    federation_enabled: data.federation_enabled,
    captcha_enabled: data.captcha_enabled,
    captcha_difficulty: data.captcha_difficulty.clone(),
    reports_email_admins: data.reports_email_admins,
    default_post_listing_mode: data.default_post_listing_mode,
    oauth_registration: data.oauth_registration,
    post_upvotes: data.post_upvotes,
    post_downvotes: data.post_downvotes,
    comment_upvotes: data.comment_upvotes,
    comment_downvotes: data.comment_downvotes,
    disallow_nsfw_content: data.disallow_nsfw_content,
    disable_email_notifications: data.disable_email_notifications,
    suggested_communities: data.suggested_communities,
    ..Default::default()
  };

  let update_local_site = LocalSite::update(&mut context.pool(), &local_site_form)
    .await
    .ok();

  let local_site_rate_limit_form = LocalSiteRateLimitUpdateForm {
    message_max_requests: data.rate_limit_message_max_requests,
    message_interval_seconds: not_zero(data.rate_limit_message_interval_seconds),
    post_max_requests: data.rate_limit_post_max_requests,
    post_interval_seconds: not_zero(data.rate_limit_post_interval_seconds),
    register_max_requests: data.rate_limit_register_max_requests,
    register_interval_seconds: not_zero(data.rate_limit_register_interval_seconds),
    image_max_requests: data.rate_limit_image_max_requests,
    image_interval_seconds: not_zero(data.rate_limit_image_interval_seconds),
    comment_max_requests: data.rate_limit_comment_max_requests,
    comment_interval_seconds: not_zero(data.rate_limit_comment_interval_seconds),
    search_max_requests: data.rate_limit_search_max_requests,
    search_interval_seconds: not_zero(data.rate_limit_search_interval_seconds),
    import_user_settings_max_requests: data.rate_limit_import_user_settings_max_requests,
    import_user_settings_interval_seconds: not_zero(
      data.rate_limit_import_user_settings_interval_seconds,
    ),
    updated_at: Some(Some(Utc::now())),
  };

  LocalSiteRateLimit::update(&mut context.pool(), &local_site_rate_limit_form)
    .await
    .ok();

  if let Some(url_blocklist) = data.blocked_urls.clone() {
    // If this validation changes it must be synced with
    // lemmy_utils::utils::markdown::create_url_blocklist_test_regex_set.
    let parsed_urls = check_urls_are_valid(&url_blocklist)?;
    LocalSiteUrlBlocklist::replace(&mut context.pool(), parsed_urls).await?;
  }

  // TODO can't think of a better way to do this.
  // If the server suddenly requires email verification, or required applications, no old users
  // will be able to log in. It really only wants this to be a requirement for NEW signups.
  // So if it was set from false, to true, you need to update all current users columns to be
  // verified.

  let old_require_application =
    local_site.registration_mode == RegistrationMode::RequireApplication;
  let new_require_application = update_local_site
    .as_ref()
    .map(|ols| ols.registration_mode == RegistrationMode::RequireApplication)
    .unwrap_or(false);
  if !old_require_application && new_require_application {
    LocalUser::set_all_users_registration_applications_accepted(&mut context.pool()).await?;
  }

  let new_require_email_verification = update_local_site
    .as_ref()
    .map(|ols| ols.require_email_verification)
    .unwrap_or(false);
  if !local_site.require_email_verification && new_require_email_verification {
    LocalUser::set_all_users_email_verified(&mut context.pool()).await?;
  }

  let site_view = SiteView::read_local(&mut context.pool()).await?;

  let rate_limit_config =
    local_site_rate_limit_to_rate_limit_config(&site_view.local_site_rate_limit);
  context.rate_limit_cell().set_config(rate_limit_config);

  Ok(Json(SiteResponse { site_view }))
}

fn validate_update_payload(local_site: &LocalSite, edit_site: &EditSite) -> LemmyResult<()> {
  // Check that the slur regex compiles, and return the regex if valid...
  // Prioritize using new slur regex from the request; if not provided, use the existing regex.
  let slur_regex = build_and_check_regex(
    edit_site
      .slur_filter_regex
      .as_deref()
      .or(local_site.slur_filter_regex.as_deref()),
  )?;

  if let Some(name) = &edit_site.name {
    // The name doesn't need to be updated, but if provided it cannot be blanked out...
    site_name_length_check(name)?;
    check_slurs_opt(&edit_site.name, &slur_regex)?;
  }

  if let Some(summary) = &edit_site.summary {
    summary_length_check(summary)?;
    check_slurs_opt(&edit_site.summary, &slur_regex)?;
  }

  site_default_post_listing_type_check(&edit_site.default_post_listing_type)?;

  // Ensure that the sidebar has fewer than the max num characters...
  if let Some(sidebar) = &edit_site.sidebar {
    is_valid_body_field(sidebar, false)?;
  }

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

  use crate::site::update::validate_update_payload;
  use lemmy_db_schema::source::local_site::LocalSite;
  use lemmy_db_schema_file::enums::{ListingType, PostSortType, RegistrationMode};
  use lemmy_db_views_site::api::EditSite;
  use lemmy_utils::error::LemmyErrorType;

  #[test]
  fn test_validate_invalid_update_payload() {
    let invalid_payloads = [
      (
        "EditSite name matches LocalSite slur filter",
        LemmyErrorType::Slurs,
        &LocalSite {
          private_instance: true,
          slur_filter_regex: Some(String::from("(foo|bar)")),
          federation_enabled: false,
          registration_mode: RegistrationMode::Open,
          ..Default::default()
        },
        &EditSite {
          name: Some(String::from("foo site_name")),
          ..Default::default()
        },
      ),
      (
        "EditSite name matches new slur filter",
        LemmyErrorType::Slurs,
        &LocalSite {
          private_instance: true,
          slur_filter_regex: Some(String::from("(foo|bar)")),
          federation_enabled: false,
          registration_mode: RegistrationMode::Open,
          ..Default::default()
        },
        &EditSite {
          name: Some(String::from("zeta site_name")),
          slur_filter_regex: Some(String::from("(zeta|alpha)")),
          ..Default::default()
        },
      ),
      (
        "EditSite listing type is Subscribed, which is invalid",
        LemmyErrorType::InvalidDefaultPostListingType,
        &LocalSite {
          private_instance: true,
          federation_enabled: false,
          registration_mode: RegistrationMode::Open,
          ..Default::default()
        },
        &EditSite {
          name: Some(String::from("site_name")),
          default_post_listing_type: Some(ListingType::Subscribed),
          ..Default::default()
        },
      ),
      (
        "EditSite requires application, but neither it nor LocalSite has an application question",
        LemmyErrorType::ApplicationQuestionRequired,
        &LocalSite {
          private_instance: true,
          federation_enabled: false,
          registration_mode: RegistrationMode::Open,
          ..Default::default()
        },
        &EditSite {
          name: Some(String::from("site_name")),
          registration_mode: Some(RegistrationMode::RequireApplication),
          ..Default::default()
        },
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
        &LocalSite {
          private_instance: true,
          federation_enabled: false,
          registration_mode: RegistrationMode::Open,
          ..Default::default()
        },
        &EditSite::default(),
      ),
      (
        "EditSite allows clearing and changing values",
        &LocalSite {
          private_instance: true,
          federation_enabled: false,
          registration_mode: RegistrationMode::Open,
          ..Default::default()
        },
        &EditSite {
          name: Some(String::from("site_name")),
          sidebar: Some(String::new()),
          summary: Some(String::new()),
          application_question: Some(String::new()),
          private_instance: Some(false),
          default_post_listing_type: Some(ListingType::All),
          default_post_sort_type: Some(PostSortType::Active),
          slur_filter_regex: Some(String::new()),
          registration_mode: Some(RegistrationMode::Open),
          federation_enabled: Some(true),
          ..Default::default()
        },
      ),
      (
        "EditSite name passes slur filter regex",
        &LocalSite {
          private_instance: true,
          slur_filter_regex: Some(String::from("(foo|bar)")),
          registration_mode: RegistrationMode::Open,
          federation_enabled: false,
          ..Default::default()
        },
        &EditSite {
          name: Some(String::from("foo site_name")),
          slur_filter_regex: Some(String::new()),
          ..Default::default()
        },
      ),
      (
        "LocalSite has application question and EditSite now requires applications,",
        &LocalSite {
          application_question: Some(String::from("question")),
          private_instance: true,
          federation_enabled: false,
          registration_mode: RegistrationMode::Open,
          ..Default::default()
        },
        &EditSite {
          name: Some(String::from("site_name")),
          registration_mode: Some(RegistrationMode::RequireApplication),
          ..Default::default()
        },
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
}
