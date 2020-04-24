use super::*;

#[derive(Deserialize)]
pub struct UserQuery {
  user_name: String,
}

impl ToApub<PersonExt> for User_ {
  // Turn a Lemmy Community into an ActivityPub group that can be sent out over the network.
  fn to_apub(&self, _conn: &PgConnection) -> Result<PersonExt, Error> {
    // TODO go through all these to_string and to_owned()
    let mut person = Person::default();
    let oprops: &mut ObjectProperties = person.as_mut();
    oprops
      .set_context_xsd_any_uri(context())?
      .set_id(self.actor_id.to_string())?
      .set_name_xsd_string(self.name.to_owned())?
      .set_published(convert_datetime(self.published))?;

    if let Some(u) = self.updated {
      oprops.set_updated(convert_datetime(u))?;
    }

    if let Some(i) = &self.preferred_username {
      oprops.set_name_xsd_string(i.to_owned())?;
    }

    let mut actor_props = ApActorProperties::default();

    actor_props
      .set_inbox(self.get_inbox_url())?
      .set_outbox(self.get_outbox_url())?
      .set_followers(self.get_followers_url())?
      .set_following(self.get_following_url())?
      .set_liked(self.get_liked_url())?;

    let public_key = PublicKey {
      id: format!("{}#main-key", self.actor_id),
      owner: self.actor_id.to_owned(),
      public_key_pem: self.public_key.to_owned().unwrap(),
    };

    Ok(person.extend(actor_props).extend(public_key.to_ext()))
  }
}

impl ActorType for User_ {
  fn actor_id(&self) -> String {
    self.actor_id.to_owned()
  }
}

impl FromApub<PersonExt> for UserForm {
  /// Parse an ActivityPub person received from another instance into a Lemmy user.
  fn from_apub(person: &PersonExt, _conn: &PgConnection) -> Result<Self, Error> {
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

/// Return the user json over HTTP.
pub async fn get_apub_user_http(
  info: Path<UserQuery>,
  db: DbPoolParam,
) -> Result<HttpResponse<Body>, Error> {
  let user = User_::find_by_email_or_username(&&db.get()?, &info.user_name)?;
  let u = user.to_apub(&db.get().unwrap())?;
  Ok(create_apub_response(&u))
}
