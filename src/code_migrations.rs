// This is for db migrations that require code
use diesel::{
  sql_types::{Nullable, Text},
  *,
};
use lemmy_db::{
  comment::Comment,
  community::{Community, CommunityForm},
  naive_now,
  post::Post,
  private_message::PrivateMessage,
  user::{UserForm, User_},
  Crud,
};
use lemmy_utils::{
  apub::{generate_actor_keypair, make_apub_endpoint, EndpointType},
  settings::Settings,
  LemmyError,
};
use log::info;

pub fn run_advanced_migrations(conn: &PgConnection) -> Result<(), LemmyError> {
  user_updates_2020_04_02(&conn)?;
  community_updates_2020_04_02(&conn)?;
  post_updates_2020_04_03(&conn)?;
  comment_updates_2020_04_03(&conn)?;
  private_message_updates_2020_05_05(&conn)?;
  post_thumbnail_url_updates_2020_07_27(&conn)?;

  Ok(())
}

fn user_updates_2020_04_02(conn: &PgConnection) -> Result<(), LemmyError> {
  use lemmy_db::schema::user_::dsl::*;

  info!("Running user_updates_2020_04_02");

  // Update the actor_id, private_key, and public_key, last_refreshed_at
  let incorrect_users = user_
    .filter(actor_id.like("changeme_%"))
    .filter(local.eq(true))
    .load::<User_>(conn)?;

  sql_query("alter table user_ disable trigger refresh_user").execute(conn)?;

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
      banned: cuser.banned,
      show_nsfw: cuser.show_nsfw,
      theme: cuser.theme.to_owned(),
      default_sort_type: cuser.default_sort_type,
      default_listing_type: cuser.default_listing_type,
      lang: cuser.lang.to_owned(),
      show_avatars: cuser.show_avatars,
      send_notifications_to_email: cuser.send_notifications_to_email,
      actor_id: Some(make_apub_endpoint(EndpointType::User, &cuser.name).to_string()),
      bio: Some(cuser.bio.to_owned()),
      local: cuser.local,
      private_key: Some(keypair.private_key),
      public_key: Some(keypair.public_key),
      last_refreshed_at: Some(naive_now()),
    };

    User_::update(&conn, cuser.id, &form)?;
  }

  sql_query("alter table user_ enable trigger refresh_user").execute(conn)?;

  info!("{} user rows updated.", incorrect_users.len());

  Ok(())
}

fn community_updates_2020_04_02(conn: &PgConnection) -> Result<(), LemmyError> {
  use lemmy_db::schema::community::dsl::*;

  info!("Running community_updates_2020_04_02");

  // Update the actor_id, private_key, and public_key, last_refreshed_at
  let incorrect_communities = community
    .filter(actor_id.like("changeme_%"))
    .filter(local.eq(true))
    .load::<Community>(conn)?;

  sql_query("alter table community disable trigger refresh_community").execute(conn)?;

  for ccommunity in &incorrect_communities {
    let keypair = generate_actor_keypair()?;

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
      actor_id: Some(make_apub_endpoint(EndpointType::Community, &ccommunity.name).to_string()),
      local: ccommunity.local,
      private_key: Some(keypair.private_key),
      public_key: Some(keypair.public_key),
      last_refreshed_at: Some(naive_now()),
      published: None,
      icon: Some(ccommunity.icon.to_owned()),
      banner: Some(ccommunity.banner.to_owned()),
    };

    Community::update(&conn, ccommunity.id, &form)?;
  }

  sql_query("alter table community enable trigger refresh_community").execute(conn)?;

  info!("{} community rows updated.", incorrect_communities.len());

  Ok(())
}

fn post_updates_2020_04_03(conn: &PgConnection) -> Result<(), LemmyError> {
  use lemmy_db::schema::post::dsl::*;

  info!("Running post_updates_2020_04_03");

  // Update the ap_id
  let incorrect_posts = post
    .filter(ap_id.eq("changeme_%"))
    .filter(local.eq(true))
    .load::<Post>(conn)?;

  sql_query("alter table post disable trigger refresh_post").execute(conn)?;

  for cpost in &incorrect_posts {
    let apub_id = make_apub_endpoint(EndpointType::Post, &cpost.id.to_string()).to_string();
    Post::update_ap_id(&conn, cpost.id, apub_id)?;
  }

  info!("{} post rows updated.", incorrect_posts.len());

  sql_query("alter table post enable trigger refresh_post").execute(conn)?;

  Ok(())
}

fn comment_updates_2020_04_03(conn: &PgConnection) -> Result<(), LemmyError> {
  use lemmy_db::schema::comment::dsl::*;

  info!("Running comment_updates_2020_04_03");

  // Update the ap_id
  let incorrect_comments = comment
    .filter(ap_id.eq("changeme_%"))
    .filter(local.eq(true))
    .load::<Comment>(conn)?;

  sql_query("alter table comment disable trigger refresh_comment").execute(conn)?;

  for ccomment in &incorrect_comments {
    let apub_id = make_apub_endpoint(EndpointType::Comment, &ccomment.id.to_string()).to_string();
    Comment::update_ap_id(&conn, ccomment.id, apub_id)?;
  }

  sql_query("alter table comment enable trigger refresh_comment").execute(conn)?;

  info!("{} comment rows updated.", incorrect_comments.len());

  Ok(())
}

fn private_message_updates_2020_05_05(conn: &PgConnection) -> Result<(), LemmyError> {
  use lemmy_db::schema::private_message::dsl::*;

  info!("Running private_message_updates_2020_05_05");

  // Update the ap_id
  let incorrect_pms = private_message
    .filter(ap_id.eq("changeme_%"))
    .filter(local.eq(true))
    .load::<PrivateMessage>(conn)?;

  for cpm in &incorrect_pms {
    let apub_id = make_apub_endpoint(EndpointType::PrivateMessage, &cpm.id.to_string()).to_string();
    PrivateMessage::update_ap_id(&conn, cpm.id, apub_id)?;
  }

  info!("{} private message rows updated.", incorrect_pms.len());

  Ok(())
}

fn post_thumbnail_url_updates_2020_07_27(conn: &PgConnection) -> Result<(), LemmyError> {
  use lemmy_db::schema::post::dsl::*;

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
