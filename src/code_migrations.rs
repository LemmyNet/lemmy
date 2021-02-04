// This is for db migrations that require code
use diesel::{
  sql_types::{Nullable, Text},
  *,
};
use lemmy_apub::{
  generate_apub_endpoint,
  generate_followers_url,
  generate_inbox_url,
  generate_shared_inbox_url,
  EndpointType,
};
use lemmy_db_queries::{
  source::{comment::Comment_, post::Post_, private_message::PrivateMessage_},
  Crud,
};
use lemmy_db_schema::{
  naive_now,
  source::{
    comment::Comment,
    community::{Community, CommunityForm},
    post::Post,
    private_message::PrivateMessage,
    user::{UserForm, User_},
  },
};
use lemmy_utils::{apub::generate_actor_keypair, settings::Settings, LemmyError};
use log::info;

pub fn run_advanced_migrations(conn: &PgConnection) -> Result<(), LemmyError> {
  user_updates_2020_04_02(&conn)?;
  community_updates_2020_04_02(&conn)?;
  post_updates_2020_04_03(&conn)?;
  comment_updates_2020_04_03(&conn)?;
  private_message_updates_2020_05_05(&conn)?;
  post_thumbnail_url_updates_2020_07_27(&conn)?;
  apub_columns_2021_02_02(&conn)?;

  Ok(())
}

fn user_updates_2020_04_02(conn: &PgConnection) -> Result<(), LemmyError> {
  use lemmy_db_schema::schema::user_::dsl::*;

  info!("Running user_updates_2020_04_02");

  // Update the actor_id, private_key, and public_key, last_refreshed_at
  let incorrect_users = user_
    .filter(actor_id.like("http://changeme_%"))
    .filter(local.eq(true))
    .load::<User_>(conn)?;

  for cuser in &incorrect_users {
    let keypair = generate_actor_keypair()?;

    let form = UserForm {
      name: cuser.name.to_owned(),
      email: Some(cuser.email.to_owned()),
      matrix_user_id: Some(cuser.matrix_user_id.to_owned()),
      avatar: Some(cuser.avatar.to_owned()),
      banner: Some(cuser.banner.to_owned()),
      password_encrypted: cuser.password_encrypted.to_owned(),
      preferred_username: Some(cuser.preferred_username.to_owned()),
      published: Some(cuser.published),
      updated: None,
      admin: cuser.admin,
      banned: Some(cuser.banned),
      show_nsfw: cuser.show_nsfw,
      theme: cuser.theme.to_owned(),
      default_sort_type: cuser.default_sort_type,
      default_listing_type: cuser.default_listing_type,
      lang: cuser.lang.to_owned(),
      show_avatars: cuser.show_avatars,
      send_notifications_to_email: cuser.send_notifications_to_email,
      actor_id: Some(generate_apub_endpoint(EndpointType::User, &cuser.name)?),
      bio: Some(cuser.bio.to_owned()),
      local: cuser.local,
      private_key: Some(keypair.private_key),
      public_key: Some(keypair.public_key),
      last_refreshed_at: Some(naive_now()),
      inbox_url: None,
      shared_inbox_url: None,
    };

    User_::update(&conn, cuser.id, &form)?;
  }

  info!("{} user rows updated.", incorrect_users.len());

  Ok(())
}

fn community_updates_2020_04_02(conn: &PgConnection) -> Result<(), LemmyError> {
  use lemmy_db_schema::schema::community::dsl::*;

  info!("Running community_updates_2020_04_02");

  // Update the actor_id, private_key, and public_key, last_refreshed_at
  let incorrect_communities = community
    .filter(actor_id.like("http://changeme_%"))
    .filter(local.eq(true))
    .load::<Community>(conn)?;

  for ccommunity in &incorrect_communities {
    let keypair = generate_actor_keypair()?;
    let community_actor_id = generate_apub_endpoint(EndpointType::Community, &ccommunity.name)?;

    let form = CommunityForm {
      name: ccommunity.name.to_owned(),
      title: ccommunity.title.to_owned(),
      description: ccommunity.description.to_owned(),
      category_id: ccommunity.category_id,
      creator_id: ccommunity.creator_id,
      removed: None,
      deleted: None,
      nsfw: ccommunity.nsfw,
      updated: None,
      actor_id: Some(community_actor_id.to_owned()),
      local: ccommunity.local,
      private_key: Some(keypair.private_key),
      public_key: Some(keypair.public_key),
      last_refreshed_at: Some(naive_now()),
      published: None,
      icon: Some(ccommunity.icon.to_owned()),
      banner: Some(ccommunity.banner.to_owned()),
      followers_url: None,
      inbox_url: None,
      shared_inbox_url: None,
    };

    Community::update(&conn, ccommunity.id, &form)?;
  }

  info!("{} community rows updated.", incorrect_communities.len());

  Ok(())
}

fn post_updates_2020_04_03(conn: &PgConnection) -> Result<(), LemmyError> {
  use lemmy_db_schema::schema::post::dsl::*;

  info!("Running post_updates_2020_04_03");

  // Update the ap_id
  let incorrect_posts = post
    .filter(ap_id.eq("http://changeme_%"))
    .filter(local.eq(true))
    .load::<Post>(conn)?;

  for cpost in &incorrect_posts {
    let apub_id = generate_apub_endpoint(EndpointType::Post, &cpost.id.to_string())?;
    Post::update_ap_id(&conn, cpost.id, apub_id)?;
  }

  info!("{} post rows updated.", incorrect_posts.len());

  Ok(())
}

fn comment_updates_2020_04_03(conn: &PgConnection) -> Result<(), LemmyError> {
  use lemmy_db_schema::schema::comment::dsl::*;

  info!("Running comment_updates_2020_04_03");

  // Update the ap_id
  let incorrect_comments = comment
    .filter(ap_id.eq("http://changeme_%"))
    .filter(local.eq(true))
    .load::<Comment>(conn)?;

  for ccomment in &incorrect_comments {
    let apub_id = generate_apub_endpoint(EndpointType::Comment, &ccomment.id.to_string())?;
    Comment::update_ap_id(&conn, ccomment.id, apub_id)?;
  }

  info!("{} comment rows updated.", incorrect_comments.len());

  Ok(())
}

fn private_message_updates_2020_05_05(conn: &PgConnection) -> Result<(), LemmyError> {
  use lemmy_db_schema::schema::private_message::dsl::*;

  info!("Running private_message_updates_2020_05_05");

  // Update the ap_id
  let incorrect_pms = private_message
    .filter(ap_id.eq("http://changeme_%"))
    .filter(local.eq(true))
    .load::<PrivateMessage>(conn)?;

  for cpm in &incorrect_pms {
    let apub_id = generate_apub_endpoint(EndpointType::PrivateMessage, &cpm.id.to_string())?;
    PrivateMessage::update_ap_id(&conn, cpm.id, apub_id)?;
  }

  info!("{} private message rows updated.", incorrect_pms.len());

  Ok(())
}

fn post_thumbnail_url_updates_2020_07_27(conn: &PgConnection) -> Result<(), LemmyError> {
  use lemmy_db_schema::schema::post::dsl::*;

  info!("Running post_thumbnail_url_updates_2020_07_27");

  let domain_prefix = format!(
    "{}/pictrs/image/",
    Settings::get().get_protocol_and_hostname(),
  );

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
fn apub_columns_2021_02_02(conn: &PgConnection) -> Result<(), LemmyError> {
  info!("Running apub_columns_2021_02_02");
  {
    use lemmy_db_schema::schema::user_::dsl::*;
    let users = user_
      .filter(inbox_url.eq("http://changeme_%"))
      .load::<User_>(conn)?;

    for u in &users {
      let inbox_url_ = generate_inbox_url(&u.actor_id)?;
      let shared_inbox_url_ = generate_shared_inbox_url(&u.actor_id)?;
      diesel::update(user_.find(u.id))
        .set((
          inbox_url.eq(inbox_url_),
          shared_inbox_url.eq(shared_inbox_url_),
        ))
        .get_result::<User_>(conn)?;
    }
  }

  {
    use lemmy_db_schema::schema::community::dsl::*;
    let communities = community
      .filter(inbox_url.eq("http://changeme_%"))
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
