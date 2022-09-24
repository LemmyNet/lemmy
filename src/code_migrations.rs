// This is for db migrations that require code
use activitypub_federation::core::signatures::generate_actor_keypair;
use diesel::{
  sql_types::{Nullable, Text},
  *,
};
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
    comment::Comment,
    community::{Community, CommunityForm},
    person::{Person, PersonForm},
    post::Post,
    private_message::PrivateMessage,
    site::{Site, SiteForm},
  },
  traits::Crud,
  utils::naive_now,
};
use lemmy_utils::error::LemmyError;
use std::default::Default;
use tracing::info;
use url::Url;

pub fn run_advanced_migrations(
  conn: &mut PgConnection,
  protocol_and_hostname: &str,
) -> Result<(), LemmyError> {
  user_updates_2020_04_02(conn, protocol_and_hostname)?;
  community_updates_2020_04_02(conn, protocol_and_hostname)?;
  post_updates_2020_04_03(conn, protocol_and_hostname)?;
  comment_updates_2020_04_03(conn, protocol_and_hostname)?;
  private_message_updates_2020_05_05(conn, protocol_and_hostname)?;
  post_thumbnail_url_updates_2020_07_27(conn, protocol_and_hostname)?;
  apub_columns_2021_02_02(conn)?;
  instance_actor_2022_01_28(conn, protocol_and_hostname)?;
  regenerate_public_keys_2022_07_05(conn)?;

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

    let form = PersonForm {
      name: cperson.name.to_owned(),
      actor_id: Some(generate_local_apub_endpoint(
        EndpointType::Person,
        &cperson.name,
        protocol_and_hostname,
      )?),
      private_key: Some(Some(keypair.private_key)),
      public_key: Some(keypair.public_key),
      last_refreshed_at: Some(naive_now()),
      ..PersonForm::default()
    };

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

    let form = CommunityForm {
      name: ccommunity.name.to_owned(),
      title: ccommunity.title.to_owned(),
      description: Some(ccommunity.description.to_owned()),
      hidden: Some(false),
      actor_id: Some(community_actor_id.to_owned()),
      local: Some(ccommunity.local),
      private_key: Some(Some(keypair.private_key)),
      public_key: Some(keypair.public_key),
      last_refreshed_at: Some(naive_now()),
      icon: Some(ccommunity.icon.to_owned()),
      banner: Some(ccommunity.banner.to_owned()),
      ..Default::default()
    };

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
    Post::update_ap_id(conn, cpost.id, apub_id)?;
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
    Comment::update_ap_id(conn, ccomment.id, apub_id)?;
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
    PrivateMessage::update_ap_id(conn, cpm.id, apub_id)?;
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
  if let Ok(site) = Site::read_local_site(conn) {
    // if site already has public key, we dont need to do anything here
    if !site.public_key.is_empty() {
      return Ok(());
    }
    let key_pair = generate_actor_keypair()?;
    let actor_id = Url::parse(protocol_and_hostname)?;
    let site_form = SiteForm {
      name: site.name,
      actor_id: Some(actor_id.clone().into()),
      last_refreshed_at: Some(naive_now()),
      inbox_url: Some(generate_site_inbox_url(&actor_id.into())?),
      private_key: Some(Some(key_pair.private_key)),
      public_key: Some(key_pair.public_key),
      ..Default::default()
    };
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
      let form = CommunityForm {
        name: community_.name,
        title: community_.title,
        public_key: Some(key_pair.public_key),
        private_key: Some(Some(key_pair.private_key)),
        ..Default::default()
      };
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
      let form = PersonForm {
        name: person_.name,
        public_key: Some(key_pair.public_key),
        private_key: Some(Some(key_pair.private_key)),
        ..Default::default()
      };
      Person::update(conn, person_.id, &form)?;
    }
  }
  Ok(())
}
