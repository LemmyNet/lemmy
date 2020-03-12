use crate::apub::make_apub_endpoint;
use crate::convert_datetime;
use crate::db::establish_unpooled_connection;
use crate::db::user::User_;
use activitystreams::{actor::apub::Person, context, object::properties::ObjectProperties};
use actix_web::body::Body;
use actix_web::web::Path;
use actix_web::HttpResponse;
use failure::Error;
use serde::Deserialize;

impl User_ {
  pub fn as_person(&self) -> Result<Person, Error> {
    let base_url = make_apub_endpoint("u", &self.name);

    let mut person = Person::default();
    let oprops: &mut ObjectProperties = person.as_mut();
    oprops
      .set_context_xsd_any_uri(context())?
      .set_id(base_url.to_string())?
      .set_published(convert_datetime(self.published))?;

    if let Some(u) = self.updated {
      oprops.set_updated(convert_datetime(u))?;
    }

    if let Some(i) = &self.preferred_username {
      oprops.set_name_xsd_string(i.to_owned())?;
    }

    person
      .ap_actor_props
      .set_inbox(format!("{}/inbox", &base_url))?
      .set_outbox(format!("{}/outbox", &base_url))?
      .set_following(format!("{}/following", &base_url))?
      .set_liked(format!("{}/liked", &base_url))?;

    Ok(person)
  }
}

#[derive(Deserialize)]
pub struct UserQuery {
  user_name: String,
}

pub async fn get_apub_user(info: Path<UserQuery>) -> Result<HttpResponse<Body>, Error> {
  let connection = establish_unpooled_connection();

  if let Ok(user) = User_::find_by_email_or_username(&connection, &info.user_name) {
    Ok(
      HttpResponse::Ok()
        .content_type("application/activity+json")
        .body(serde_json::to_string(&user.as_person()?).unwrap()),
    )
  } else {
    Ok(HttpResponse::NotFound().finish())
  }
}
