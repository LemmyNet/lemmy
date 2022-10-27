// This is for db migrations that require code
use activitypub_federation::core::signatures::generate_actor_keypair;
use diesel::{
  sql_types::{Nullable, Text},
  *,
};
use lemmy_api_common::lemmy_db_views::structs::SiteView;
use lemmy_apub::{
  generate_followers_url,
  generate_inbox_url,
  generate_local_apub_endpoint,
  generate_shared_inbox_url,
  generate_site_inbox_url,
  EndpointType,
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
  utils::naive_now,
};
use lemmy_utils::{error::LemmyError, settings::structs::Settings};
use tracing::info;
use url::Url;

pub fn run_advanced_migrations(
  conn: &mut PgConnection,
  settings: &Settings,
) -> Result<(), LemmyError> {
  let protocol_and_hostname = &settings.get_protocol_and_hostname();
  user_updates_2020_04_02(conn, protocol_and_hostname)?;
  community_updates_2020_04_02(conn, protocol_and_hostname)?;
  post_updates_2020_04_03(conn, protocol_and_hostname)?;
  comment_updates_2020_04_03(conn, protocol_and_hostname)?;
  private_message_updates_2020_05_05(conn, protocol_and_hostname)?;
  post_thumbnail_url_updates_2020_07_27(conn, protocol_and_hostname)?;
  apub_columns_2021_02_02(conn)?;
  instance_actor_2022_01_28(conn, protocol_and_hostname)?;
  regenerate_public_keys_2022_07_05(conn)?;
  initialize_local_site_2022_10_10(conn, settings)?;

  Ok(())
}

fn user_updates_2020_04_02(
  conn: &mut PgConnection,
  protocol_and_hostname: &str,
) -> Result<(), LemmyError> {
  use lemmy_db_schema::schema::person::dsl::*;

  info!("Running user_updates_2020_04_02");

  // Update the actor_id, private_key, and public_key, last_refreshed_at
  let incorrect_persons = person
    .filter(actor_id.like("http://changeme%"))
    .filter(local.eq(true))
    .load::<Person>(conn)?;

  for cperson in &incorrect_persons {
    let keypair = generate_actor_keypair()?;

    let form = PersonUpdateForm::builder()
      .actor_id(Some(generate_local_apub_endpoint(
        EndpointType::Person,
        &cperson.name,
        protocol_and_hostname,
      )?))
      .private_key(Some(Some(keypair.private_key)))
      .public_key(Some(keypair.public_key))
      .last_refreshed_at(Some(naive_now()))
      .build();

    Person::update(conn, cperson.id, &form)?;
  }

  info!("{} person rows updated.", incorrect_persons.len());

  Ok(())
}

fn community_updates_2020_04_02(
  conn: &mut PgConnection,
  protocol_and_hostname: &str,
) -> Result<(), LemmyError> {
  use lemmy_db_schema::schema::community::dsl::*;

  info!("Running community_updates_2020_04_02");

  // Update the actor_id, private_key, and public_key, last_refreshed_at
  let incorrect_communities = community
    .filter(actor_id.like("http://changeme%"))
    .filter(local.eq(true))
    .load::<Community>(conn)?;

  for ccommunity in &incorrect_communities {
    let keypair = generate_actor_keypair()?;
    let community_actor_id = generate_local_apub_endpoint(
      EndpointType::Community,
      &ccommunity.name,
      protocol_and_hostname,
    )?;

    let form = CommunityUpdateForm::builder()
      .actor_id(Some(community_actor_id.to_owned()))
      .private_key(Some(Some(keypair.private_key)))
      .public_key(Some(keypair.public_key))
      .last_refreshed_at(Some(naive_now()))
      .build();

    Community::update(conn, ccommunity.id, &form)?;
  }

  info!("{} community rows updated.", incorrect_communities.len());

  Ok(())
}

fn post_updates_2020_04_03(
  conn: &mut PgConnection,
  protocol_and_hostname: &str,
) -> Result<(), LemmyError> {
  use lemmy_db_schema::schema::post::dsl::*;

  info!("Running post_updates_2020_04_03");

  // Update the ap_id
  let incorrect_posts = post
    .filter(ap_id.like("http://changeme%"))
    .filter(local.eq(true))
    .load::<Post>(conn)?;

  for cpost in &incorrect_posts {
    let apub_id = generate_local_apub_endpoint(
      EndpointType::Post,
      &cpost.id.to_string(),
      protocol_and_hostname,
    )?;
    Post::update(
      conn,
      cpost.id,
      &PostUpdateForm::builder().ap_id(Some(apub_id)).build(),
    )?;
  }

  info!("{} post rows updated.", incorrect_posts.len());

  Ok(())
}

fn comment_updates_2020_04_03(
  conn: &mut PgConnection,
  protocol_and_hostname: &str,
) -> Result<(), LemmyError> {
  use lemmy_db_schema::schema::comment::dsl::*;

  info!("Running comment_updates_2020_04_03");

  // Update the ap_id
  let incorrect_comments = comment
    .filter(ap_id.like("http://changeme%"))
    .filter(local.eq(true))
    .load::<Comment>(conn)?;

  for ccomment in &incorrect_comments {
    let apub_id = generate_local_apub_endpoint(
      EndpointType::Comment,
      &ccomment.id.to_string(),
      protocol_and_hostname,
    )?;
    Comment::update(
      conn,
      ccomment.id,
      &CommentUpdateForm::builder().ap_id(Some(apub_id)).build(),
    )?;
  }

  info!("{} comment rows updated.", incorrect_comments.len());

  Ok(())
}

fn private_message_updates_2020_05_05(
  conn: &mut PgConnection,
  protocol_and_hostname: &str,
) -> Result<(), LemmyError> {
  use lemmy_db_schema::schema::private_message::dsl::*;

  info!("Running private_message_updates_2020_05_05");

  // Update the ap_id
  let incorrect_pms = private_message
    .filter(ap_id.like("http://changeme%"))
    .filter(local.eq(true))
    .load::<PrivateMessage>(conn)?;

  for cpm in &incorrect_pms {
    let apub_id = generate_local_apub_endpoint(
      EndpointType::PrivateMessage,
      &cpm.id.to_string(),
      protocol_and_hostname,
    )?;
    PrivateMessage::update(
      conn,
      cpm.id,
      &PrivateMessageUpdateForm::builder()
        .ap_id(Some(apub_id))
        .build(),
    )?;
  }

  info!("{} private message rows updated.", incorrect_pms.len());

  Ok(())
}

fn post_thumbnail_url_updates_2020_07_27(
  conn: &mut PgConnection,
  protocol_and_hostname: &str,
) -> Result<(), LemmyError> {
  use lemmy_db_schema::schema::post::dsl::*;

  info!("Running post_thumbnail_url_updates_2020_07_27");

  let domain_prefix = format!("{}/pictrs/image/", protocol_and_hostname,);

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
    .get_results::<Post>(conn)?;

  info!("{} Post thumbnail_url rows updated.", res.len());

  Ok(())
}

/// We are setting inbox and follower URLs for local and remote actors alike, because for now
/// all federated instances are also Lemmy and use the same URL scheme.
fn apub_columns_2021_02_02(conn: &mut PgConnection) -> Result<(), LemmyError> {
  info!("Running apub_columns_2021_02_02");
  {
    use lemmy_db_schema::schema::person::dsl::*;
    let persons = person
      .filter(inbox_url.like("http://changeme%"))
      .load::<Person>(conn)?;

    for p in &persons {
      let inbox_url_ = generate_inbox_url(&p.actor_id)?;
      let shared_inbox_url_ = generate_shared_inbox_url(&p.actor_id)?;
      diesel::update(person.find(p.id))
        .set((
          inbox_url.eq(inbox_url_),
          shared_inbox_url.eq(shared_inbox_url_),
        ))
        .get_result::<Person>(conn)?;
    }
  }

  {
    use lemmy_db_schema::schema::community::dsl::*;
    let communities = community
      .filter(inbox_url.like("http://changeme%"))
      .load::<Community>(conn)?;

    for c in &communities {
      let followers_url_ = generate_followers_url(&c.actor_id)?;
      let inbox_url_ = generate_inbox_url(&c.actor_id)?;
      let shared_inbox_url_ = generate_shared_inbox_url(&c.actor_id)?;
      diesel::update(community.find(c.id))
        .set((
          followers_url.eq(followers_url_),
          inbox_url.eq(inbox_url_),
          shared_inbox_url.eq(shared_inbox_url_),
        ))
        .get_result::<Community>(conn)?;
    }
  }

  Ok(())
}

/// Site object turns into an actor, so that things like instance description can be federated. This
/// means we need to add actor columns to the site table, and initialize them with correct values.
/// Before this point, there is only a single value in the site table which refers to the local
/// Lemmy instance, so thats all we need to update.
fn instance_actor_2022_01_28(
  conn: &mut PgConnection,
  protocol_and_hostname: &str,
) -> Result<(), LemmyError> {
  info!("Running instance_actor_2021_09_29");
  if let Ok(site_view) = SiteView::read_local(conn) {
    let site = site_view.site;
    // if site already has public key, we dont need to do anything here
    if !site.public_key.is_empty() {
      return Ok(());
    }
    let key_pair = generate_actor_keypair()?;
    let actor_id = Url::parse(protocol_and_hostname)?;
    let site_form = SiteUpdateForm::builder()
      .actor_id(Some(actor_id.clone().into()))
      .last_refreshed_at(Some(naive_now()))
      .inbox_url(Some(generate_site_inbox_url(&actor_id.into())?))
      .private_key(Some(Some(key_pair.private_key)))
      .public_key(Some(key_pair.public_key))
      .build();
    Site::update(conn, site.id, &site_form)?;
  }
  Ok(())
}

/// Fix for bug #2347, which can result in community/person public keys being overwritten with
/// empty string when the database value is updated. We go through all actors, and if the public
/// key field is empty, generate a new keypair. It would be possible to regenerate only the pubkey,
/// but thats more complicated and has no benefit, as federation is already broken for these actors.
/// https://github.com/LemmyNet/lemmy/issues/2347
fn regenerate_public_keys_2022_07_05(conn: &mut PgConnection) -> Result<(), LemmyError> {
  info!("Running regenerate_public_keys_2022_07_05");

  {
    // update communities with empty pubkey
    use lemmy_db_schema::schema::community::dsl::*;
    let communities: Vec<Community> = community
      .filter(local.eq(true))
      .filter(public_key.eq(""))
      .load::<Community>(conn)?;
    for community_ in communities {
      info!(
        "local community {} has empty public key field, regenerating key",
        community_.name
      );
      let key_pair = generate_actor_keypair()?;
      let form = CommunityUpdateForm::builder()
        .public_key(Some(key_pair.public_key))
        .private_key(Some(Some(key_pair.private_key)))
        .build();
      Community::update(conn, community_.id, &form)?;
    }
  }

  {
    // update persons with empty pubkey
    use lemmy_db_schema::schema::person::dsl::*;
    let persons = person
      .filter(local.eq(true))
      .filter(public_key.eq(""))
      .load::<Person>(conn)?;
    for person_ in persons {
      info!(
        "local user {} has empty public key field, regenerating key",
        person_.name
      );
      let key_pair = generate_actor_keypair()?;
      let form = PersonUpdateForm::builder()
        .public_key(Some(key_pair.public_key))
        .private_key(Some(Some(key_pair.private_key)))
        .build();
      Person::update(conn, person_.id, &form)?;
    }
  }
  Ok(())
}

/// This ensures that your local site is initialized and exists.
///
/// If a site already exists, the DB migration should generate a local_site row.
/// This will only be run for brand new sites.
fn initialize_local_site_2022_10_10(
  conn: &mut PgConnection,
  settings: &Settings,
) -> Result<(), LemmyError> {
  info!("Running initialize_local_site_2022_10_10");

  // Check to see if local_site exists
  if LocalSite::read(conn).is_ok() {
    return Ok(());
  }
  info!("No Local Site found, creating it.");

  let domain = settings
    .get_hostname_without_port()
    .expect("must have domain");

  // Upsert this to the instance table
  let instance = Instance::create(conn, &domain)?;

  if let Some(setup) = &settings.setup {
    let person_keypair = generate_actor_keypair()?;
    let person_actor_id = generate_local_apub_endpoint(
      EndpointType::Person,
      &setup.admin_username,
      &settings.get_protocol_and_hostname(),
    )?;

    // Register the user if there's a site setup
    let person_form = PersonInsertForm::builder()
      .name(setup.admin_username.to_owned())
      .admin(Some(true))
      .instance_id(instance.id)
      .actor_id(Some(person_actor_id.clone()))
      .private_key(Some(person_keypair.private_key))
      .public_key(person_keypair.public_key)
      .inbox_url(Some(generate_inbox_url(&person_actor_id)?))
      .shared_inbox_url(Some(generate_shared_inbox_url(&person_actor_id)?))
      .build();
    let person_inserted = Person::create(conn, &person_form)?;

    let local_user_form = LocalUserInsertForm::builder()
      .person_id(person_inserted.id)
      .password_encrypted(setup.admin_password.to_owned())
      .email(setup.admin_email.to_owned())
      .build();
    LocalUser::create(conn, &local_user_form)?;
  };

  // Add an entry for the site table
  let site_key_pair = generate_actor_keypair()?;
  let site_actor_id = Url::parse(&settings.get_protocol_and_hostname())?;

  let site_form = SiteInsertForm::builder()
    .name(
      settings
        .setup
        .to_owned()
        .map(|s| s.site_name)
        .unwrap_or_else(|| "New Site".to_string()),
    )
    .instance_id(instance.id)
    .actor_id(Some(site_actor_id.clone().into()))
    .last_refreshed_at(Some(naive_now()))
    .inbox_url(Some(generate_site_inbox_url(&site_actor_id.into())?))
    .private_key(Some(site_key_pair.private_key))
    .public_key(Some(site_key_pair.public_key))
    .build();
  let site = Site::create(conn, &site_form)?;

  // Finally create the local_site row
  let local_site_form = LocalSiteInsertForm::builder()
    .site_id(site.id)
    .site_setup(Some(settings.setup.is_some()))
    .build();
  let local_site = LocalSite::create(conn, &local_site_form)?;

  // Create the rate limit table
  let local_site_rate_limit_form = LocalSiteRateLimitInsertForm::builder()
    // TODO these have to be set, because the database defaults are too low for the federation
    // tests to pass, and there's no way to live update the rate limits without restarting the
    // server.
    // This can be removed once live rate limits are enabled.
    .message(Some(999))
    .post(Some(999))
    .register(Some(999))
    .image(Some(999))
    .comment(Some(999))
    .search(Some(999))
    .local_site_id(local_site.id)
    .build();
  LocalSiteRateLimit::create(conn, &local_site_rate_limit_form)?;

  Ok(())
}
