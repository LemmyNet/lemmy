// This is for db migrations that require code
use super::comment::Comment;
use super::community::{Community, CommunityForm};
use super::post::Post;
use super::user::{UserForm, User_};
use super::*;
use crate::apub::{gen_keypair_str, make_apub_endpoint, EndpointType};
use crate::naive_now;
use log::info;

pub fn run_advanced_migrations(conn: &PgConnection) -> Result<(), Error> {
  user_updates_2020_04_02(conn)?;
  community_updates_2020_04_02(conn)?;
  post_updates_2020_04_03(conn)?;
  comment_updates_2020_04_03(conn)?;

  Ok(())
}

fn user_updates_2020_04_02(conn: &PgConnection) -> Result<(), Error> {
  use crate::schema::user_::dsl::*;

  info!("Running user_updates_2020_04_02");

  // Update the actor_id, private_key, and public_key, last_refreshed_at
  let incorrect_users = user_
    .filter(actor_id.eq("changeme"))
    .filter(local.eq(true))
    .load::<User_>(conn)?;

  for cuser in &incorrect_users {
    let (user_public_key, user_private_key) = gen_keypair_str();

    let form = UserForm {
      name: cuser.name.to_owned(),
      fedi_name: cuser.fedi_name.to_owned(),
      email: cuser.email.to_owned(),
      matrix_user_id: cuser.matrix_user_id.to_owned(),
      avatar: cuser.avatar.to_owned(),
      password_encrypted: cuser.password_encrypted.to_owned(),
      preferred_username: cuser.preferred_username.to_owned(),
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
      actor_id: make_apub_endpoint(EndpointType::User, &cuser.name).to_string(),
      bio: cuser.bio.to_owned(),
      local: cuser.local,
      private_key: Some(user_private_key),
      public_key: Some(user_public_key),
      last_refreshed_at: Some(naive_now()),
    };

    User_::update(&conn, cuser.id, &form)?;
  }

  info!("{} user rows updated.", incorrect_users.len());

  Ok(())
}

fn community_updates_2020_04_02(conn: &PgConnection) -> Result<(), Error> {
  use crate::schema::community::dsl::*;

  info!("Running community_updates_2020_04_02");

  // Update the actor_id, private_key, and public_key, last_refreshed_at
  let incorrect_communities = community
    .filter(actor_id.eq("changeme"))
    .filter(local.eq(true))
    .load::<Community>(conn)?;

  for ccommunity in &incorrect_communities {
    let (community_public_key, community_private_key) = gen_keypair_str();

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
      actor_id: make_apub_endpoint(EndpointType::Community, &ccommunity.name).to_string(),
      local: ccommunity.local,
      private_key: Some(community_private_key),
      public_key: Some(community_public_key),
      last_refreshed_at: Some(naive_now()),
    };

    Community::update(&conn, ccommunity.id, &form)?;
  }

  info!("{} community rows updated.", incorrect_communities.len());

  Ok(())
}

fn post_updates_2020_04_03(conn: &PgConnection) -> Result<(), Error> {
  use crate::schema::post::dsl::*;

  info!("Running post_updates_2020_04_03");

  // Update the ap_id
  let incorrect_posts = post
    .filter(ap_id.eq("changeme"))
    .filter(local.eq(true))
    .load::<Post>(conn)?;

  for cpost in &incorrect_posts {
    Post::update_ap_id(&conn, cpost.id)?;
  }

  info!("{} post rows updated.", incorrect_posts.len());

  Ok(())
}

fn comment_updates_2020_04_03(conn: &PgConnection) -> Result<(), Error> {
  use crate::schema::comment::dsl::*;

  info!("Running comment_updates_2020_04_03");

  // Update the ap_id
  let incorrect_comments = comment
    .filter(ap_id.eq("changeme"))
    .filter(local.eq(true))
    .load::<Comment>(conn)?;

  for ccomment in &incorrect_comments {
    Comment::update_ap_id(&conn, ccomment.id)?;
  }

  info!("{} comment rows updated.", incorrect_comments.len());

  Ok(())
}
