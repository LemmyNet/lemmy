use crate::apub::{create_apub_response, make_apub_endpoint, EndpointType};
use crate::convert_datetime;
use crate::db::user::User_;
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
  let base_url = make_apub_endpoint(EndpointType::User, &user.name);

  let mut person = Person::default();
  let oprops: &mut ObjectProperties = person.as_mut();
  oprops
    .set_context_xsd_any_uri(context())?
    .set_id(base_url.to_string())?
    .set_published(convert_datetime(user.published))?;

  if let Some(u) = user.updated {
    oprops.set_updated(convert_datetime(u))?;
  }

  if let Some(i) = &user.preferred_username {
    oprops.set_name_xsd_string(i.to_owned())?;
  }

  let mut actor_props = ApActorProperties::default();

  actor_props
    .set_inbox(format!("{}/inbox", &base_url))?
    .set_outbox(format!("{}/outbox", &base_url))?
    .set_following(format!("{}/following", &base_url))?
    .set_liked(format!("{}/liked", &base_url))?;

  Ok(create_apub_response(&person.extend(actor_props)))
}
