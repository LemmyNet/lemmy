use crate::apub::signatures::PublicKey;
use crate::apub::{create_apub_response, PersonExt};
use crate::db::user::{UserForm, User_};
use crate::{convert_datetime, naive_now};
use activitystreams::{
  actor::{properties::ApActorProperties, Person},
  context,
  ext::Extensible,
  object::properties::ObjectProperties,
};
use actix_web::body::Body;
use actix_web::web::Path;
use actix_web::HttpResponse;
use actix_web::{web, Result};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserQuery {
  user_name: String,
}

pub async fn get_apub_user(
  info: Path<UserQuery>,
  db: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<HttpResponse<Body>, Error> {
  let user = User_::find_by_email_or_username(&&db.get()?, &info.user_name)?;

  let mut person = Person::default();
  let oprops: &mut ObjectProperties = person.as_mut();
  oprops
    .set_context_xsd_any_uri(context())?
    .set_id(user.actor_id.to_string())?
    .set_name_xsd_string(user.name.to_owned())?
    .set_published(convert_datetime(user.published))?;

  if let Some(u) = user.updated {
    oprops.set_updated(convert_datetime(u))?;
  }

  if let Some(i) = &user.preferred_username {
    oprops.set_name_xsd_string(i.to_owned())?;
  }

  let mut actor_props = ApActorProperties::default();

  actor_props
    .set_inbox(format!("{}/inbox", &user.actor_id))?
    .set_outbox(format!("{}/outbox", &user.actor_id))?
    .set_following(format!("{}/following", &user.actor_id))?
    .set_liked(format!("{}/liked", &user.actor_id))?;

  let public_key = PublicKey {
    id: format!("{}#main-key", user.actor_id),
    owner: user.actor_id.to_owned(),
    public_key_pem: user.public_key.unwrap(),
  };

  Ok(create_apub_response(
    &person.extend(actor_props).extend(public_key.to_ext()),
  ))
}

impl UserForm {
  pub fn from_person(person: &PersonExt) -> Result<Self, Error> {
    let oprops = &person.base.base.object_props;
    let aprops = &person.base.extension;
    let public_key: &PublicKey = &person.extension.public_key;

    Ok(UserForm {
      name: oprops.get_name_xsd_string().unwrap().to_string(),
      preferred_username: aprops.get_preferred_username().map(|u| u.to_string()),
      password_encrypted: "".to_string(),
      admin: false,
      banned: false,
      email: None,
      avatar: None, // -> icon, image
      updated: oprops
        .get_updated()
        .map(|u| u.as_ref().to_owned().naive_local()),
      show_nsfw: false,
      theme: "".to_string(),
      default_sort_type: 0,
      default_listing_type: 0,
      lang: "".to_string(),
      show_avatars: false,
      send_notifications_to_email: false,
      matrix_user_id: None,
      actor_id: oprops.get_id().unwrap().to_string(),
      bio: oprops.get_summary_xsd_string().map(|s| s.to_string()),
      local: false,
      private_key: None,
      public_key: Some(public_key.to_owned().public_key_pem),
      last_refreshed_at: Some(naive_now()),
    })
  }
}
