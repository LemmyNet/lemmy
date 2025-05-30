use activitypub_federation::http_signatures::generate_actor_keypair;
use chrono::Utc;
use lemmy_api_common::utils::generate_inbox_url;
use lemmy_db_schema::{
  source::{
    instance::Instance,
    local_site::{LocalSite, LocalSiteInsertForm},
    local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitInsertForm},
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
    site::{Site, SiteInsertForm},
  },
  traits::{ApubActor, Crud},
  utils::DbPool,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use tracing::info;
use url::Url;

pub async fn setup_local_site(pool: &mut DbPool<'_>, settings: &Settings) -> LemmyResult<SiteView> {
  // Check to see if local_site exists
  if let Ok(site_view) = SiteView::read_local(pool).await {
    return Ok(site_view);
  }
  info!("No Local Site found, creating it.");

  let domain = settings
    .get_hostname_without_port()
    .with_lemmy_type(LemmyErrorType::Unknown("must have domain".into()))?;

  // Upsert this to the instance table
  let instance = Instance::read_or_create(pool, domain).await?;

  if let Some(setup) = &settings.setup {
    let person_keypair = generate_actor_keypair()?;
    let person_ap_id = Person::generate_local_actor_url(&setup.admin_username, settings)?;

    // Register the user if there's a site setup
    let person_form = PersonInsertForm {
      ap_id: Some(person_ap_id.clone()),
      inbox_url: Some(generate_inbox_url()?),
      private_key: Some(person_keypair.private_key.into()),
      ..PersonInsertForm::new(
        setup.admin_username.clone(),
        person_keypair.public_key,
        instance.id,
      )
    };
    let person_inserted = Person::create(pool, &person_form).await?;

    let local_user_form = LocalUserInsertForm {
      email: setup.admin_email.clone(),
      admin: Some(true),
      ..LocalUserInsertForm::new(person_inserted.id, Some(setup.admin_password.clone()))
    };
    LocalUser::create(pool, &local_user_form, vec![]).await?;
  };

  // Add an entry for the site table
  let site_key_pair = generate_actor_keypair()?;
  let site_ap_id = Url::parse(&settings.get_protocol_and_hostname())?;

  let name = settings
    .setup
    .clone()
    .map(|s| s.site_name)
    .unwrap_or_else(|| "New Site".to_string());
  let site_form = SiteInsertForm {
    ap_id: Some(site_ap_id.clone().into()),
    last_refreshed_at: Some(Utc::now()),
    inbox_url: Some(generate_inbox_url()?),
    private_key: Some(site_key_pair.private_key),
    public_key: Some(site_key_pair.public_key),
    ..SiteInsertForm::new(name, instance.id)
  };
  let site = Site::create(pool, &site_form).await?;

  // Finally create the local_site row
  let local_site_form = LocalSiteInsertForm {
    site_setup: Some(settings.setup.is_some()),
    ..LocalSiteInsertForm::new(site.id)
  };
  let local_site = LocalSite::create(pool, &local_site_form).await?;

  // Create the rate limit table
  let local_site_rate_limit_form = if cfg!(debug_assertions) {
    LocalSiteRateLimitInsertForm {
      message: Some(999),
      post: Some(999),
      register: Some(999),
      image: Some(999),
      comment: Some(999),
      search: Some(999),
      ..LocalSiteRateLimitInsertForm::new(local_site.id)
    }
  } else {
    LocalSiteRateLimitInsertForm::new(local_site.id)
  };
  // TODO these have to be set, because the database defaults are too low for the federation
  // tests to pass, and there's no way to live update the rate limits without restarting the
  // server.
  // This can be removed once live rate limits are enabled.
  LocalSiteRateLimit::create(pool, &local_site_rate_limit_form).await?;

  SiteView::read_local(pool).await
}
