use activitypub_federation::http_signatures::generate_actor_keypair;
use chrono::Utc;
use diesel::{
  dsl::{exists, not, select},
  query_builder::AsQuery,
};
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_api_utils::utils::generate_inbox_url;
use lemmy_db_schema::{
  source::{
    instance::Instance,
    local_site::{LocalSite, LocalSiteInsertForm},
    local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitInsertForm},
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
    site::{Site, SiteInsertForm},
  },
  traits::ApubActor,
};
use lemmy_db_schema_file::schema::local_site;
use lemmy_db_views_site::SiteView;
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  sensitive::SensitiveString,
  traits::Crud,
};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use rand::{Rng, distr::Alphanumeric};
use tracing::info;
use url::Url;

pub async fn setup_local_site(pool: &mut DbPool<'_>, settings: &Settings) -> LemmyResult<SiteView> {
  let conn = &mut get_conn(pool).await?;
  // Check to see if local_site exists, without the cache wrapper
  if select(not(exists(local_site::table.as_query())))
    .get_result(conn)
    .await?
  {
    info!("No Local Site found, creating it.");

    let domain = settings
      .get_hostname_without_port()
      .with_lemmy_type(LemmyErrorType::Unknown("must have domain".into()))?;

    conn
      .run_transaction(|conn| {
        async move {
          // Upsert this to the instance table
          let instance = Instance::read_or_create(&mut conn.into(), &domain).await?;

          if let Some(setup) = &settings.setup {
            let person_keypair = generate_actor_keypair()?;
            let person_ap_id = Person::generate_local_actor_url(&setup.admin_username, settings)?;

            // Register the user if there's a site setup
            let person_form = PersonInsertForm {
              ap_id: Some(person_ap_id.clone()),
              inbox_url: Some(generate_inbox_url()?),
              private_key: Some(person_keypair.private_key),
              ..PersonInsertForm::new(
                setup.admin_username.clone(),
                person_keypair.public_key,
                instance.id,
              )
            };
            let person_inserted = Person::create(&mut conn.into(), &person_form).await?;

            let local_user_form = LocalUserInsertForm {
              email: setup.admin_email.clone(),
              admin: Some(true),
              ..LocalUserInsertForm::new(person_inserted.id, Some(setup.admin_password.clone()))
            };
            LocalUser::create(&mut conn.into(), &local_user_form, vec![]).await?;
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
          let site = Site::create(&mut conn.into(), &site_form).await?;
          // create multi-comm follower account
          let r: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(11)
            .map(char::from)
            .collect();
          let name = format!("multicomm{}", r);
          let form = PersonInsertForm {
            private_key: site.private_key.map(SensitiveString::into_inner),
            inbox_url: Some(site.inbox_url),
            bot_account: Some(true),
            ..PersonInsertForm::new(name, site.public_key, instance.id)
          };
          let multi_comm_follower = Person::create(&mut conn.into(), &form).await?;

          // Finally create the local_site row
          let local_site_form = LocalSiteInsertForm {
            site_setup: Some(settings.setup.is_some()),
            multi_comm_follower: Some(multi_comm_follower.id),
            ..LocalSiteInsertForm::new(site.id)
          };
          let local_site = LocalSite::create(&mut conn.into(), &local_site_form).await?;

          // Create the rate limit table
          let local_site_rate_limit_form = LocalSiteRateLimitInsertForm::new(local_site.id);
          LocalSiteRateLimit::create(&mut conn.into(), &local_site_rate_limit_form).await?;
          Ok(())
        }
        .scope_boxed()
      })
      .await?;
  }

  SiteView::read_local(pool).await
}
