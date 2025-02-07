// This is for db migrations that require code
use activitypub_federation::http_signatures::generate_actor_keypair;
use chrono::Utc;
use diesel::{
  sql_types::{Nullable, Text},
  ExpressionMethods,
  IntoSql,
  QueryDsl,
  TextExpressionMethods,
};
use diesel_async::RunQueryDsl;
use lemmy_api_common::{
  lemmy_db_views::structs::SiteView,
  utils::{generate_followers_url, generate_inbox_url},
};
use lemmy_db_schema::{
  source::{
    comment::{Comment, CommentUpdateForm},
    community::{Community, CommunityUpdateForm},
    instance::Instance,
    local_site::{LocalSite, LocalSiteInsertForm},
    local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitInsertForm},
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm, PersonUpdateForm},
    post::{Post, PostUpdateForm},
    private_message::{PrivateMessage, PrivateMessageUpdateForm},
    site::{Site, SiteInsertForm, SiteUpdateForm},
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use tracing::info;
use url::Url;

pub async fn run_advanced_migrations(
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> LemmyResult<()> {
  let protocol_and_hostname = &settings.get_protocol_and_hostname();
  user_updates_2020_04_02(pool, settings).await?;
  community_updates_2020_04_02(pool, settings).await?;
  post_updates_2020_04_03(pool, settings).await?;
  comment_updates_2020_04_03(pool, settings).await?;
  private_message_updates_2020_05_05(pool, settings).await?;
  post_thumbnail_url_updates_2020_07_27(pool, protocol_and_hostname).await?;
  apub_columns_2021_02_02(pool).await?;
  instance_actor_2022_01_28(pool, protocol_and_hostname).await?;
  regenerate_public_keys_2022_07_05(pool).await?;
  initialize_local_site_2022_10_10(pool, settings).await?;

  Ok(())
}

async fn user_updates_2020_04_02(pool: &mut DbPool<'_>, settings: &Settings) -> LemmyResult<()> {
  use lemmy_db_schema::schema::person::dsl::{ap_id, local, person};
  let conn = &mut get_conn(pool).await?;

  info!("Running user_updates_2020_04_02");

  // Update the ap_id, private_key, and public_key, last_refreshed_at
  let incorrect_persons = person
    .filter(ap_id.like("http://changeme%"))
    .filter(local.eq(true))
    .load::<Person>(conn)
    .await?;

  for cperson in &incorrect_persons {
    let keypair = generate_actor_keypair()?;

    let form = PersonUpdateForm {
      ap_id: Some(Person::local_url(&cperson.name, settings)?),
      private_key: Some(Some(keypair.private_key)),
      public_key: Some(keypair.public_key),
      last_refreshed_at: Some(Utc::now()),
      ..Default::default()
    };

    Person::update(pool, cperson.id, &form).await?;
  }

  info!("{} person rows updated.", incorrect_persons.len());

  Ok(())
}

async fn community_updates_2020_04_02(
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> LemmyResult<()> {
  use lemmy_db_schema::schema::community::dsl::{ap_id, community, local};
  let conn = &mut get_conn(pool).await?;

  info!("Running community_updates_2020_04_02");

  // Update the ap_id, private_key, and public_key, last_refreshed_at
  let incorrect_communities = community
    .filter(ap_id.like("http://changeme%"))
    .filter(local.eq(true))
    .load::<Community>(conn)
    .await?;

  for ccommunity in &incorrect_communities {
    let keypair = generate_actor_keypair()?;
    let community_ap_id = Community::local_url(&ccommunity.name, settings)?;

    let form = CommunityUpdateForm {
      ap_id: Some(community_ap_id.clone()),
      private_key: Some(Some(keypair.private_key)),
      public_key: Some(keypair.public_key),
      last_refreshed_at: Some(Utc::now()),
      ..Default::default()
    };

    Community::update(pool, ccommunity.id, &form).await?;
  }

  info!("{} community rows updated.", incorrect_communities.len());

  Ok(())
}

async fn post_updates_2020_04_03(pool: &mut DbPool<'_>, settings: &Settings) -> LemmyResult<()> {
  use lemmy_db_schema::schema::post::dsl::{ap_id, local, post};
  let conn = &mut get_conn(pool).await?;

  info!("Running post_updates_2020_04_03");

  // Update the ap_id
  let incorrect_posts = post
    .filter(ap_id.like("http://changeme%"))
    .filter(local.eq(true))
    .load::<Post>(conn)
    .await?;

  for cpost in &incorrect_posts {
    let apub_id = cpost.local_url(settings)?;
    Post::update(
      pool,
      cpost.id,
      &PostUpdateForm {
        ap_id: Some(apub_id),
        ..Default::default()
      },
    )
    .await?;
  }

  info!("{} post rows updated.", incorrect_posts.len());

  Ok(())
}

async fn comment_updates_2020_04_03(pool: &mut DbPool<'_>, settings: &Settings) -> LemmyResult<()> {
  use lemmy_db_schema::schema::comment::dsl::{ap_id, comment, local};
  let conn = &mut get_conn(pool).await?;

  info!("Running comment_updates_2020_04_03");

  // Update the ap_id
  let incorrect_comments = comment
    .filter(ap_id.like("http://changeme%"))
    .filter(local.eq(true))
    .load::<Comment>(conn)
    .await?;

  for ccomment in &incorrect_comments {
    let apub_id = ccomment.local_url(settings)?;
    Comment::update(
      pool,
      ccomment.id,
      &CommentUpdateForm {
        ap_id: Some(apub_id),
        ..Default::default()
      },
    )
    .await?;
  }

  info!("{} comment rows updated.", incorrect_comments.len());

  Ok(())
}

async fn private_message_updates_2020_05_05(
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> LemmyResult<()> {
  use lemmy_db_schema::schema::private_message::dsl::{ap_id, local, private_message};
  let conn = &mut get_conn(pool).await?;

  info!("Running private_message_updates_2020_05_05");

  // Update the ap_id
  let incorrect_pms = private_message
    .filter(ap_id.like("http://changeme%"))
    .filter(local.eq(true))
    .load::<PrivateMessage>(conn)
    .await?;

  for cpm in &incorrect_pms {
    let apub_id = cpm.local_url(settings)?;
    PrivateMessage::update(
      pool,
      cpm.id,
      &PrivateMessageUpdateForm {
        ap_id: Some(apub_id),
        ..Default::default()
      },
    )
    .await?;
  }

  info!("{} private message rows updated.", incorrect_pms.len());

  Ok(())
}

async fn post_thumbnail_url_updates_2020_07_27(
  pool: &mut DbPool<'_>,
  protocol_and_hostname: &str,
) -> LemmyResult<()> {
  use lemmy_db_schema::schema::post::dsl::{post, thumbnail_url};
  let conn = &mut get_conn(pool).await?;

  info!("Running post_thumbnail_url_updates_2020_07_27");

  let domain_prefix = format!("{protocol_and_hostname}/pictrs/image/",);

  let incorrect_thumbnails = post.filter(thumbnail_url.not_like("http%"));

  // Prepend the rows with the update
  let res = diesel::update(incorrect_thumbnails)
    .set(
      thumbnail_url.eq(
        domain_prefix
          .into_sql::<Nullable<Text>>()
          .concat(thumbnail_url),
      ),
    )
    .get_results::<Post>(conn)
    .await?;

  info!("{} Post thumbnail_url rows updated.", res.len());

  Ok(())
}

/// We are setting inbox and follower URLs for local and remote actors alike, because for now
/// all federated instances are also Lemmy and use the same URL scheme.
async fn apub_columns_2021_02_02(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  let conn = &mut get_conn(pool).await?;
  info!("Running apub_columns_2021_02_02");
  {
    use lemmy_db_schema::schema::person::dsl::{inbox_url, person};
    let persons = person
      .filter(inbox_url.like("http://changeme%"))
      .load::<Person>(conn)
      .await?;

    for p in &persons {
      let inbox_url_ = generate_inbox_url()?;
      diesel::update(person.find(p.id))
        .set((inbox_url.eq(inbox_url_),))
        .get_result::<Person>(conn)
        .await?;
    }
  }

  {
    use lemmy_db_schema::schema::community::dsl::{community, followers_url, inbox_url};
    let communities = community
      .filter(inbox_url.like("http://changeme%"))
      .load::<Community>(conn)
      .await?;

    for c in &communities {
      let followers_url_ = generate_followers_url(&c.ap_id)?;
      let inbox_url_ = generate_inbox_url()?;
      diesel::update(community.find(c.id))
        .set((followers_url.eq(followers_url_), inbox_url.eq(inbox_url_)))
        .get_result::<Community>(conn)
        .await?;
    }
  }

  Ok(())
}

/// Site object turns into an actor, so that things like instance description can be federated. This
/// means we need to add actor columns to the site table, and initialize them with correct values.
/// Before this point, there is only a single value in the site table which refers to the local
/// Lemmy instance, so thats all we need to update.
async fn instance_actor_2022_01_28(
  pool: &mut DbPool<'_>,
  protocol_and_hostname: &str,
) -> LemmyResult<()> {
  info!("Running instance_actor_2021_09_29");
  if let Ok(site_view) = SiteView::read_local(pool).await {
    let site = site_view.site;
    // if site already has public key, we dont need to do anything here
    if !site.public_key.is_empty() {
      return Ok(());
    }
    let key_pair = generate_actor_keypair()?;
    let ap_id = Url::parse(protocol_and_hostname)?;
    let site_form = SiteUpdateForm {
      ap_id: Some(ap_id.clone().into()),
      last_refreshed_at: Some(Utc::now()),
      inbox_url: Some(generate_inbox_url()?),
      private_key: Some(Some(key_pair.private_key)),
      public_key: Some(key_pair.public_key),
      ..Default::default()
    };
    Site::update(pool, site.id, &site_form).await?;
  }
  Ok(())
}

/// Fix for bug #2347, which can result in community/person public keys being overwritten with
/// empty string when the database value is updated. We go through all actors, and if the public
/// key field is empty, generate a new keypair. It would be possible to regenerate only the pubkey,
/// but thats more complicated and has no benefit, as federation is already broken for these actors.
/// https://github.com/LemmyNet/lemmy/issues/2347
async fn regenerate_public_keys_2022_07_05(pool: &mut DbPool<'_>) -> LemmyResult<()> {
  let conn = &mut get_conn(pool).await?;
  info!("Running regenerate_public_keys_2022_07_05");

  {
    // update communities with empty pubkey
    use lemmy_db_schema::schema::community::dsl::{community, local, public_key};
    let communities: Vec<Community> = community
      .filter(local.eq(true))
      .filter(public_key.eq(""))
      .load::<Community>(conn)
      .await?;
    for community_ in communities {
      info!(
        "local community {} has empty public key field, regenerating key",
        community_.name
      );
      let key_pair = generate_actor_keypair()?;
      let form = CommunityUpdateForm {
        public_key: Some(key_pair.public_key),
        private_key: Some(Some(key_pair.private_key)),
        ..Default::default()
      };
      Community::update(&mut conn.into(), community_.id, &form).await?;
    }
  }

  {
    // update persons with empty pubkey
    use lemmy_db_schema::schema::person::dsl::{local, person, public_key};
    let persons = person
      .filter(local.eq(true))
      .filter(public_key.eq(""))
      .load::<Person>(conn)
      .await?;
    for person_ in persons {
      info!(
        "local user {} has empty public key field, regenerating key",
        person_.name
      );
      let key_pair = generate_actor_keypair()?;
      let form = PersonUpdateForm {
        public_key: Some(key_pair.public_key),
        private_key: Some(Some(key_pair.private_key)),
        ..Default::default()
      };
      Person::update(pool, person_.id, &form).await?;
    }
  }
  Ok(())
}

/// This ensures that your local site is initialized and exists.
///
/// If a site already exists, the DB migration should generate a local_site row.
/// This will only be run for brand new sites.
async fn initialize_local_site_2022_10_10(
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> LemmyResult<()> {
  info!("Running initialize_local_site_2022_10_10");

  // Check to see if local_site exists
  if LocalSite::read(pool).await.is_ok() {
    return Ok(());
  }
  info!("No Local Site found, creating it.");

  let domain = settings
    .get_hostname_without_port()
    .with_lemmy_type(LemmyErrorType::Unknown("must have domain".into()))?;

  // Upsert this to the instance table
  let instance = Instance::read_or_create(pool, domain).await?;

  if let Some(setup) = &settings.setup {
    let person_keypair = generate_actor_keypair()?;
    let person_ap_id = Person::local_url(&setup.admin_username, settings)?;

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
  let local_site_rate_limit_form = LocalSiteRateLimitInsertForm {
    message: Some(999),
    post: Some(999),
    register: Some(999),
    image: Some(999),
    comment: Some(999),
    search: Some(999),
    ..LocalSiteRateLimitInsertForm::new(local_site.id)
  };
  // TODO these have to be set, because the database defaults are too low for the federation
  // tests to pass, and there's no way to live update the rate limits without restarting the
  // server.
  // This can be removed once live rate limits are enabled.
  LocalSiteRateLimit::create(pool, &local_site_rate_limit_form).await?;

  Ok(())
}
