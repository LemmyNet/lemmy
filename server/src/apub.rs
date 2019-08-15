extern crate activitypub;
use self::activitypub::{context, actor::Person};
use crate::db::user::User_;

impl User_ {
  pub fn person(&self) -> Person {
    use crate::{Settings, to_datetime_utc};
    let base_url = &format!("{}/user/{}", Settings::get().api_endpoint(), self.name);
    let mut person  = Person::default();
    person.object_props.set_context_object(context()).ok();
    person.object_props.set_id_string(base_url.to_string()).ok();
    person.object_props.set_name_string(self.name.to_owned()).ok();
    person.object_props.set_published_utctime(to_datetime_utc(self.published)).ok();
    if let Some(i) = self.updated {
      person.object_props.set_updated_utctime(to_datetime_utc(i)).ok();
    }
    // person.object_props.summary = self.summary;

    person.ap_actor_props.set_inbox_string(format!("{}/inbox", &base_url)).ok();
    person.ap_actor_props.set_outbox_string(format!("{}/outbox", &base_url)).ok();
    person.ap_actor_props.set_following_string(format!("{}/following", &base_url)).ok();
    person.ap_actor_props.set_liked_string(format!("{}/liked", &base_url)).ok();
    if let Some(i) = &self.preferred_username {
      person.ap_actor_props.set_preferred_username_string(i.to_string()).ok();
    }

    person
  }
}

#[cfg(test)]
mod tests {
  use super::User_;
  use crate::naive_now;

  #[test]
  fn test_person() {
    let expected_user = User_ {
      id: 52,
      name: "thom".into(),
      fedi_name: "rrf".into(),
      preferred_username: None,
      password_encrypted: "here".into(),
      email: None,
      icon: None,
      published: naive_now(),
      admin: false,
      banned: false,
      updated: None,
      show_nsfw: false,
    };

    let person = expected_user.person();
    assert_eq!("rrr/api/v1/user/thom", person.object_props.id_string().unwrap());
    let json = serde_json::to_string_pretty(&person).unwrap();
    println!("{}", json);

  }
}

