use crate::apub::make_apub_endpoint;
use crate::db::user::User_;
use crate::to_datetime_utc;
use activitypub::{actor::Person, context};

impl User_ {
  pub fn as_person(&self) -> Person {
    let base_url = make_apub_endpoint("user", &self.name);
    let mut person = Person::default();
    person.object_props.set_context_object(context()).ok();
    person.object_props.set_id_string(base_url.to_string()).ok();
    person
      .object_props
      .set_name_string(self.name.to_owned())
      .ok();
    person
      .object_props
      .set_published_utctime(to_datetime_utc(self.published))
      .ok();
    if let Some(updated) = self.updated {
      person
        .object_props
        .set_updated_utctime(to_datetime_utc(updated))
        .ok();
    }

    person
      .ap_actor_props
      .set_inbox_string(format!("{}/inbox", &base_url))
      .ok();
    person
      .ap_actor_props
      .set_outbox_string(format!("{}/outbox", &base_url))
      .ok();
    person
      .ap_actor_props
      .set_following_string(format!("{}/following", &base_url))
      .ok();
    person
      .ap_actor_props
      .set_liked_string(format!("{}/liked", &base_url))
      .ok();
    if let Some(i) = &self.preferred_username {
      person
        .ap_actor_props
        .set_preferred_username_string(i.to_string())
        .ok();
    }

    person
  }
}
