use super::not_zero;
use crate::site::{application_question_check, site_default_post_listing_type_check};
use activitypub_federation::{config::Data, http_signatures::generate_actor_keypair};
use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{
    generate_inbox_url,
    get_url_blocklist,
    is_admin,
    local_site_rate_limit_to_rate_limit_config,
    process_markdown_opt,
    slur_regex,
  },
};
use lemmy_db_schema::source::{
  local_site::{LocalSite, LocalSiteUpdateForm},
  local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitUpdateForm},
  site::{Site, SiteUpdateForm},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::{
  SiteView,
  api::{CreateSite, SiteResponse},
};
use lemmy_diesel_utils::{dburl::DbUrl, traits::Crud, utils::diesel_string_update};
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  utils::{
    slurs::check_slurs,
    validation::{
      build_and_check_regex,
      is_valid_body_field,
      site_name_length_check,
      summary_length_check,
    },
  },
};
use url::Url;

pub async fn create_site(
  Json(data): Json<CreateSite>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SiteResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;

  // Make sure user is an admin; other types of users should not create site data...
  is_admin(&local_user_view)?;

  validate_create_payload(&local_site, &data)?;

  let ap_id: DbUrl = Url::parse(&context.settings().get_protocol_and_hostname())?.into();
  let inbox_url = Some(generate_inbox_url()?);
  let keypair = generate_actor_keypair()?;

  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  let sidebar = process_markdown_opt(&data.sidebar, &slur_regex, &url_blocklist, &context).await?;

  let site_form = SiteUpdateForm {
    name: Some(data.name.clone()),
    sidebar: diesel_string_update(sidebar.as_deref()),
    summary: diesel_string_update(data.summary.as_deref()),
    ap_id: Some(ap_id),
    last_refreshed_at: Some(Utc::now()),
    inbox_url,
    private_key: Some(Some(keypair.private_key)),
    public_key: Some(keypair.public_key),
    content_warning: diesel_string_update(data.content_warning.as_deref()),
    ..Default::default()
  };

  let site_id = local_site.site_id;

  Site::update(&mut context.pool(), site_id, &site_form).await?;

  let local_site_form = LocalSiteUpdateForm {
    // Set the site setup to true
    site_setup: Some(true),
    registration_mode: data.registration_mode,
    community_creation_admin_only: data.community_creation_admin_only,
    require_email_verification: data.require_email_verification,
    application_question: diesel_string_update(data.application_question.as_deref()),
    private_instance: data.private_instance,
    default_theme: data.default_theme.clone(),
    default_post_listing_type: data.default_post_listing_type,
    default_post_sort_type: data.default_post_sort_type,
    default_comment_sort_type: data.default_comment_sort_type,
    legal_information: diesel_string_update(data.legal_information.as_deref()),
    application_email_admins: data.application_email_admins,
    updated_at: Some(Some(Utc::now())),
    slur_filter_regex: diesel_string_update(data.slur_filter_regex.as_deref()),
    federation_enabled: data.federation_enabled,
    captcha_enabled: data.captcha_enabled,
    captcha_difficulty: data.captcha_difficulty.clone(),
    default_post_listing_mode: data.default_post_listing_mode,
    post_upvotes: data.post_upvotes,
    post_downvotes: data.post_downvotes,
    comment_upvotes: data.comment_upvotes,
    comment_downvotes: data.comment_downvotes,
    disallow_nsfw_content: data.disallow_nsfw_content,
    disable_email_notifications: data.disable_email_notifications,
    suggested_communities: data.suggested_communities,
    ..Default::default()
  };

  LocalSite::update(&mut context.pool(), &local_site_form).await?;

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

  LocalSiteRateLimit::update(&mut context.pool(), &local_site_rate_limit_form).await?;

  let site_view = SiteView::read_local(&mut context.pool()).await?;

  let rate_limit_config =
    local_site_rate_limit_to_rate_limit_config(&site_view.local_site_rate_limit);
  context.rate_limit_cell().set_config(rate_limit_config);

  Ok(Json(SiteResponse { site_view }))
}

fn validate_create_payload(local_site: &LocalSite, create_site: &CreateSite) -> LemmyResult<()> {
  // Make sure the site hasn't already been set up...
  if local_site.site_setup {
    Err(LemmyErrorType::AlreadyExists)?
  };

  // Check that the slur regex compiles, and returns the regex if valid...
  // Prioritize using new slur regex from the request; if not provided, use the existing regex.
  let slur_regex = build_and_check_regex(
    create_site
      .slur_filter_regex
      .as_deref()
      .or(local_site.slur_filter_regex.as_deref()),
  )?;

  site_name_length_check(&create_site.name)?;
  check_slurs(&create_site.name, &slur_regex)?;

  if let Some(desc) = &create_site.summary {
    summary_length_check(desc)?;
    check_slurs(desc, &slur_regex)?;
  }

  site_default_post_listing_type_check(&create_site.default_post_listing_type)?;

  // Ensure that the sidebar has fewer than the max num characters...
  if let Some(sidebar) = &create_site.sidebar {
    is_valid_body_field(sidebar, false)?;
  }

  application_question_check(
    &local_site.application_question,
    &create_site.application_question,
    create_site
      .registration_mode
      .unwrap_or(local_site.registration_mode),
  )
}

#[cfg(test)]
mod tests {
  use crate::site::create::validate_create_payload;
  use lemmy_db_schema::source::local_site::LocalSite;
  use lemmy_db_schema_file::enums::{ListingType, PostSortType, RegistrationMode};
  use lemmy_db_views_site::api::CreateSite;
  use lemmy_utils::error::LemmyErrorType;

  #[test]
  fn test_validate_invalid_create_payload() {
    let invalid_payloads = [
      (
        "CreateSite attempted on set up LocalSite",
        LemmyErrorType::AlreadyExists,
        &LocalSite {
          site_setup: true,
          private_instance: true,
          federation_enabled: false,
          registration_mode: RegistrationMode::Open,
          ..Default::default()
        },
        &CreateSite {
          name: String::from("site_name"),
          ..Default::default()
        },
      ),
      (
        "CreateSite name matches LocalSite slur filter",
        LemmyErrorType::Slurs,
        &LocalSite {
          site_setup: false,
          private_instance: true,
          slur_filter_regex: Some(String::from("(foo|bar)")),
          federation_enabled: false,
          registration_mode: RegistrationMode::Open,
          ..Default::default()
        },
        &CreateSite {
          name: String::from("foo site_name"),
          ..Default::default()
        },
      ),
      (
        "CreateSite name matches new slur filter",
        LemmyErrorType::Slurs,
        &LocalSite {
          site_setup: false,
          private_instance: true,
          slur_filter_regex: Some(String::from("(foo|bar)")),
          federation_enabled: false,
          registration_mode: RegistrationMode::Open,
          ..Default::default()
        },
        &CreateSite {
          name: String::from("zeta site_name"),
          slur_filter_regex: Some(String::from("(zeta|alpha)")),
          ..Default::default()
        },
      ),
      (
        "CreateSite listing type is Subscribed, which is invalid",
        LemmyErrorType::InvalidDefaultPostListingType,
        &LocalSite {
          site_setup: false,
          private_instance: true,
          federation_enabled: false,
          registration_mode: RegistrationMode::Open,
          ..Default::default()
        },
        &CreateSite {
          name: String::from("site_name"),
          default_post_listing_type: Some(ListingType::Subscribed),
          ..Default::default()
        },
      ),
      (
        "CreateSite requires application, but neither it nor LocalSite has an application question",
        LemmyErrorType::ApplicationQuestionRequired,
        &LocalSite {
          site_setup: false,
          private_instance: true,
          federation_enabled: false,
          registration_mode: RegistrationMode::Open,
          ..Default::default()
        },
        &CreateSite {
          name: String::from("site_name"),
          registration_mode: Some(RegistrationMode::RequireApplication),
          ..Default::default()
        },
      ),
    ];

    invalid_payloads.iter().enumerate().for_each(
      |(
         idx,
         &(reason, ref expected_err, local_site, create_site),
       )| {
        match validate_create_payload(
          local_site,
          create_site,
        ) {
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
  fn test_validate_valid_create_payload() {
    let valid_payloads = [
      (
        "No changes between LocalSite and CreateSite",
        &LocalSite {
          site_setup: false,
          private_instance: true,
          federation_enabled: false,
          registration_mode: RegistrationMode::Open,
          ..Default::default()
        },
        &CreateSite {
          name: String::from("site_name"),
          ..Default::default()
        },
      ),
      (
        "CreateSite allows clearing and changing values",
        &LocalSite {
          site_setup: false,
          private_instance: true,
          federation_enabled: false,
          registration_mode: RegistrationMode::Open,
          ..Default::default()
        },
        &CreateSite {
          name: String::from("site_name"),
          sidebar: Some(String::new()),
          summary: Some(String::new()),
          application_question: Some(String::new()),
          private_instance: Some(false),
          default_post_listing_type: Some(ListingType::All),
          default_post_sort_type: Some(PostSortType::Active),
          slur_filter_regex: Some(String::new()),
          federation_enabled: Some(true),
          registration_mode: Some(RegistrationMode::Open),
          ..Default::default()
        },
      ),
      (
        "CreateSite clears existing slur filter regex",
        &LocalSite {
          site_setup: false,
          private_instance: true,
          slur_filter_regex: Some(String::from("(foo|bar)")),
          federation_enabled: false,
          registration_mode: RegistrationMode::Open,
          ..Default::default()
        },
        &CreateSite {
          name: String::from("foo site_name"),
          slur_filter_regex: Some(String::new()),
          ..Default::default()
        },
      ),
      (
        "LocalSite has application question and CreateSite now requires applications,",
        &LocalSite {
          site_setup: false,
          application_question: Some(String::from("question")),
          private_instance: true,
          federation_enabled: false,
          registration_mode: RegistrationMode::Open,
          ..Default::default()
        },
        &CreateSite {
          name: String::from("site_name"),
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
          validate_create_payload(local_site, edit_site).is_ok(),
          "Got Err, but should have got Ok for reason: {}. valid_payloads.nth({})",
          reason,
          idx
        );
      })
  }
}
