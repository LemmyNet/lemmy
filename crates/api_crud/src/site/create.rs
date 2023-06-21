use crate::{site::check_application_question, PerformCrud};
use activitypub_federation::http_signatures::generate_actor_keypair;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  site::{CreateSite, SiteResponse},
  utils::{
    generate_site_inbox_url,
    is_admin,
    local_site_rate_limit_to_rate_limit_config,
    local_site_to_slur_regex,
    local_user_view_from_jwt,
    site_description_length_check,
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
  utils::{diesel_option_overwrite, diesel_option_overwrite_to_url, naive_now},
};
use lemmy_db_views::structs::SiteView;
use lemmy_utils::{
  error::LemmyError,
  utils::{
    slurs::{check_slurs, check_slurs_opt},
    validation::{check_site_visibility_valid, is_valid_body_field},
  },
};
use url::Url;

#[async_trait::async_trait(?Send)]
impl PerformCrud for CreateSite {
  type Response = SiteResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<SiteResponse, LemmyError> {
    let data: &CreateSite = self;

    let local_site = LocalSite::read(context.pool()).await?;

    if local_site.site_setup {
      return Err(LemmyError::from_message("site_already_exists"));
    };

    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    check_site_visibility_valid(
      local_site.private_instance,
      local_site.federation_enabled,
      &data.private_instance,
      &data.federation_enabled,
    )?;

    let sidebar = diesel_option_overwrite(&data.sidebar);
    let description = diesel_option_overwrite(&data.description);
    let icon = diesel_option_overwrite_to_url(&data.icon)?;
    let banner = diesel_option_overwrite_to_url(&data.banner)?;

    let slur_regex = local_site_to_slur_regex(&local_site);
    check_slurs(&data.name, &slur_regex)?;
    check_slurs_opt(&data.description, &slur_regex)?;

    if let Some(Some(desc)) = &description {
      site_description_length_check(desc)?;
    }

    is_valid_body_field(&data.sidebar)?;

    let application_question = diesel_option_overwrite(&data.application_question);
    check_application_question(
      &application_question,
      data
        .registration_mode
        .unwrap_or(local_site.registration_mode),
    )?;

    let actor_id: DbUrl = Url::parse(&context.settings().get_protocol_and_hostname())?.into();
    let inbox_url = Some(generate_site_inbox_url(&actor_id)?);
    let keypair = generate_actor_keypair()?;
    let site_form = SiteUpdateForm::builder()
      .name(Some(data.name.clone()))
      .sidebar(sidebar)
      .description(description)
      .icon(icon)
      .banner(banner)
      .actor_id(Some(actor_id))
      .last_refreshed_at(Some(naive_now()))
      .inbox_url(inbox_url)
      .private_key(Some(Some(keypair.private_key)))
      .public_key(Some(keypair.public_key))
      .build();

    let site_id = local_site.site_id;

    Site::update(context.pool(), site_id, &site_form).await?;

    let local_site_form = LocalSiteUpdateForm::builder()
      // Set the site setup to true
      .site_setup(Some(true))
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
      .build();

    LocalSite::update(context.pool(), &local_site_form).await?;

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

    LocalSiteRateLimit::update(context.pool(), &local_site_rate_limit_form).await?;

    let site_view = SiteView::read_local(context.pool()).await?;

    let new_taglines = data.taglines.clone();
    let taglines = Tagline::replace(context.pool(), local_site.id, new_taglines).await?;

    let rate_limit_config =
      local_site_rate_limit_to_rate_limit_config(&site_view.local_site_rate_limit);
    context
      .settings_updated_channel()
      .send(rate_limit_config)
      .await?;

    Ok(SiteResponse {
      site_view,
      taglines,
    })
  }
}
