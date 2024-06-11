use crate::site::{application_question_check, site_default_post_listing_type_check};
use activitypub_federation::http_signatures::generate_actor_keypair;
use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  site::{CreateSite, SiteResponse},
  utils::{
    generate_shared_inbox_url,
    get_url_blocklist,
    is_admin,
    local_site_rate_limit_to_rate_limit_config,
    local_site_to_slur_regex,
    process_markdown_opt,
    proxy_image_link_api,
  },
};
use lemmy_db_schema::{
  newtypes::DbUrl,
  source::{
    local_site::{LocalSite, LocalSiteUpdateForm},
    local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitUpdateForm},
    site::{Site, SiteUpdateForm},
    tagline::Tagline,
  },
  traits::Crud,
  utils::{diesel_string_update, diesel_url_create, naive_now},
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  utils::{
    slurs::{check_slurs, check_slurs_opt},
    validation::{
      build_and_check_regex,
      check_site_visibility_valid,
      is_valid_body_field,
      site_description_length_check,
      site_name_length_check,
    },
  },
};
use url::Url;

#[tracing::instrument(skip(context))]
pub async fn create_site(
  data: Json<CreateSite>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SiteResponse>> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  // Make sure user is an admin; other types of users should not create site data...
  is_admin(&local_user_view)?;

  validate_create_payload(&local_site, &data)?;

  let keypair = generate_actor_keypair()?;

  let slur_regex = local_site_to_slur_regex(&local_site);
  let url_blocklist = get_url_blocklist(&context).await?;
  let sidebar = process_markdown_opt(&data.sidebar, &slur_regex, &url_blocklist, &context).await?;

  let icon = diesel_url_create(data.icon.as_deref())?;
  let icon = proxy_image_link_api(icon, &context).await?;

  let banner = diesel_url_create(data.banner.as_deref())?;
  let banner = proxy_image_link_api(banner, &context).await?;

  let site_form = SiteUpdateForm {
    name: Some(data.name.clone()),
    sidebar: diesel_string_update(sidebar.as_deref()),
    description: diesel_string_update(data.description.as_deref()),
    icon: Some(icon),
    banner: Some(banner),
    last_refreshed_at: Some(naive_now()),
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
    enable_downvotes: data.enable_downvotes,
    registration_mode: data.registration_mode,
    enable_nsfw: data.enable_nsfw,
    community_creation_admin_only: data.community_creation_admin_only,
    require_email_verification: data.require_email_verification,
    application_question: diesel_string_update(data.application_question.as_deref()),
    private_instance: data.private_instance,
    default_theme: data.default_theme.clone(),
    default_post_listing_type: data.default_post_listing_type,
    default_sort_type: data.default_sort_type,
    legal_information: diesel_string_update(data.legal_information.as_deref()),
    application_email_admins: data.application_email_admins,
    hide_modlog_mod_names: data.hide_modlog_mod_names,
    updated: Some(Some(naive_now())),
    slur_filter_regex: diesel_string_update(data.slur_filter_regex.as_deref()),
    actor_name_max_length: data.actor_name_max_length,
    federation_enabled: data.federation_enabled,
    captcha_enabled: data.captcha_enabled,
    captcha_difficulty: data.captcha_difficulty.clone(),
    default_post_listing_mode: data.default_post_listing_mode,
    ..Default::default()
  };

  LocalSite::update(&mut context.pool(), &local_site_form).await?;

  let local_site_rate_limit_form = LocalSiteRateLimitUpdateForm {
    message: data.rate_limit_message,
    message_per_second: data.rate_limit_message_per_second,
    post: data.rate_limit_post,
    post_per_second: data.rate_limit_post_per_second,
    register: data.rate_limit_register,
    register_per_second: data.rate_limit_register_per_second,
    image: data.rate_limit_image,
    image_per_second: data.rate_limit_image_per_second,
    comment: data.rate_limit_comment,
    comment_per_second: data.rate_limit_comment_per_second,
    search: data.rate_limit_search,
    search_per_second: data.rate_limit_search_per_second,
    ..Default::default()
  };

  LocalSiteRateLimit::update(&mut context.pool(), &local_site_rate_limit_form).await?;

  let site_view = SiteView::read_local(&mut context.pool())
    .await?
    .ok_or(LemmyErrorType::LocalSiteNotSetup)?;

  let new_taglines = data.taglines.clone();
  let taglines = Tagline::replace(&mut context.pool(), local_site.id, new_taglines).await?;

  let rate_limit_config =
    local_site_rate_limit_to_rate_limit_config(&site_view.local_site_rate_limit);
  context.rate_limit_cell().set_config(rate_limit_config);

  Ok(Json(SiteResponse {
    site_view,
    taglines,
  }))
}

fn validate_create_payload(local_site: &LocalSite, create_site: &CreateSite) -> LemmyResult<()> {
  // Make sure the site hasn't already been set up...
  if local_site.site_setup {
    Err(LemmyErrorType::SiteAlreadyExists)?
  };

  // Check that the slur regex compiles, and returns the regex if valid...
  // Prioritize using new slur regex from the request; if not provided, use the existing regex.
  let slur_regex = build_and_check_regex(
    &create_site
      .slur_filter_regex
      .as_deref()
      .or(local_site.slur_filter_regex.as_deref()),
  )?;

  site_name_length_check(&create_site.name)?;
  check_slurs(&create_site.name, &slur_regex)?;

  if let Some(desc) = &create_site.description {
    site_description_length_check(desc)?;
    check_slurs_opt(&create_site.description, &slur_regex)?;
  }

  site_default_post_listing_type_check(&create_site.default_post_listing_type)?;

  check_site_visibility_valid(
    local_site.private_instance,
    local_site.federation_enabled,
    &create_site.private_instance,
    &create_site.federation_enabled,
  )?;

  // Ensure that the sidebar has fewer than the max num characters...
  if let Some(body) = &create_site.sidebar {
    is_valid_body_field(body, false)?;
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
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use crate::site::create::validate_create_payload;
  use lemmy_api_common::site::CreateSite;
  use lemmy_db_schema::{source::local_site::LocalSite, ListingType, RegistrationMode, SortType};
  use lemmy_utils::error::LemmyErrorType;

  #[test]
  fn test_validate_invalid_create_payload() {
    let invalid_payloads = [
      (
        "CreateSite attempted on set up LocalSite",
        LemmyErrorType::SiteAlreadyExists,
        &generate_local_site(
          true,
          None::<String>,
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_create_site(
          String::from("site_name"),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          None::<SortType>,
          None::<String>,
          None::<bool>,
          None::<bool>,
          None::<String>,
          None::<RegistrationMode>,
        ),
      ),
      (
        "CreateSite name matches LocalSite slur filter",
        LemmyErrorType::Slurs,
        &generate_local_site(
          false,
          Some(String::from("(foo|bar)")),
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_create_site(
          String::from("foo site_name"),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          None::<SortType>,
          None::<String>,
          None::<bool>,
          None::<bool>,
          None::<String>,
          None::<RegistrationMode>,
        ),
      ),
      (
        "CreateSite name matches new slur filter",
        LemmyErrorType::Slurs,
        &generate_local_site(
          false,
          Some(String::from("(foo|bar)")),
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_create_site(
          String::from("zeta site_name"),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          None::<SortType>,
          Some(String::from("(zeta|alpha)")),
          None::<bool>,
          None::<bool>,
          None::<String>,
          None::<RegistrationMode>,
        ),
      ),
      (
        "CreateSite listing type is Subscribed, which is invalid",
        LemmyErrorType::InvalidDefaultPostListingType,
        &generate_local_site(
          false,
          None::<String>,
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_create_site(
          String::from("site_name"),
          None::<String>,
          None::<String>,
          Some(ListingType::Subscribed),
          None::<SortType>,
          None::<String>,
          None::<bool>,
          None::<bool>,
          None::<String>,
          None::<RegistrationMode>,
        ),
      ),
      (
        "CreateSite is both private and federated",
        LemmyErrorType::CantEnablePrivateInstanceAndFederationTogether,
        &generate_local_site(
          false,
          None::<String>,
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_create_site(
          String::from("site_name"),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          None::<SortType>,
          None::<String>,
          Some(true),
          Some(true),
          None::<String>,
          None::<RegistrationMode>,
        ),
      ),
      (
        "LocalSite is private, but CreateSite also makes it federated",
        LemmyErrorType::CantEnablePrivateInstanceAndFederationTogether,
        &generate_local_site(
          false,
          None::<String>,
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_create_site(
          String::from("site_name"),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          None::<SortType>,
          None::<String>,
          None::<bool>,
          Some(true),
          None::<String>,
          None::<RegistrationMode>,
        ),
      ),
      (
        "CreateSite requires application, but neither it nor LocalSite has an application question",
        LemmyErrorType::ApplicationQuestionRequired,
        &generate_local_site(
          false,
          None::<String>,
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_create_site(
          String::from("site_name"),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          None::<SortType>,
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
        &generate_local_site(
          false,
          None::<String>,
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_create_site(
          String::from("site_name"),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          None::<SortType>,
          None::<String>,
          None::<bool>,
          None::<bool>,
          None::<String>,
          None::<RegistrationMode>,
        ),
      ),
      (
        "CreateSite allows clearing and changing values",
        &generate_local_site(
          false,
          None::<String>,
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_create_site(
          String::from("site_name"),
          Some(String::new()),
          Some(String::new()),
          Some(ListingType::All),
          Some(SortType::Active),
          Some(String::new()),
          Some(false),
          Some(true),
          Some(String::new()),
          Some(RegistrationMode::Open),
        ),
      ),
      (
        "CreateSite clears existing slur filter regex",
        &generate_local_site(
          false,
          Some(String::from("(foo|bar)")),
          true,
          false,
          None::<String>,
          RegistrationMode::Open,
        ),
        &generate_create_site(
          String::from("foo site_name"),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          None::<SortType>,
          Some(String::new()),
          None::<bool>,
          None::<bool>,
          None::<String>,
          None::<RegistrationMode>,
        ),
      ),
      (
        "LocalSite has application question and CreateSite now requires applications,",
        &generate_local_site(
          false,
          None::<String>,
          true,
          false,
          Some(String::from("question")),
          RegistrationMode::Open,
        ),
        &generate_create_site(
          String::from("site_name"),
          None::<String>,
          None::<String>,
          None::<ListingType>,
          None::<SortType>,
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
          validate_create_payload(local_site, edit_site).is_ok(),
          "Got Err, but should have got Ok for reason: {}. valid_payloads.nth({})",
          reason,
          idx
        );
      })
  }

  fn generate_local_site(
    site_setup: bool,
    site_slur_filter_regex: Option<String>,
    site_is_private: bool,
    site_is_federated: bool,
    site_application_question: Option<String>,
    site_registration_mode: RegistrationMode,
  ) -> LocalSite {
    LocalSite {
      site_setup,
      application_question: site_application_question,
      private_instance: site_is_private,
      slur_filter_regex: site_slur_filter_regex,
      federation_enabled: site_is_federated,
      registration_mode: site_registration_mode,
      ..Default::default()
    }
  }

  // Allow the test helper function to have too many arguments.
  // It's either this or generate the entire struct each time for testing.
  #[allow(clippy::too_many_arguments)]
  fn generate_create_site(
    site_name: String,
    site_description: Option<String>,
    site_sidebar: Option<String>,
    site_listing_type: Option<ListingType>,
    site_sort_type: Option<SortType>,
    site_slur_filter_regex: Option<String>,
    site_is_private: Option<bool>,
    site_is_federated: Option<bool>,
    site_application_question: Option<String>,
    site_registration_mode: Option<RegistrationMode>,
  ) -> CreateSite {
    CreateSite {
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
      default_sort_type: site_sort_type,
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
      content_warning: None,
      default_post_listing_mode: None,
    }
  }
}
