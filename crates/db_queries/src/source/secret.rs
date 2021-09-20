use diesel::{result::Error, *};
use lemmy_db_schema::source::secret::Secret;
use lemmy_utils::settings::structs::Settings;
use std::sync::RwLock;

use crate::get_database_url_from_env;

lazy_static! {
  static ref SECRET: RwLock<Secret> = RwLock::new(init().expect("Failed to load secrets from DB."));
}

pub trait SecretSingleton {
  fn get() -> Secret;
}

impl SecretSingleton for Secret {
  /// Returns the Secret as a struct
  fn get() -> Self {
    SECRET.read().expect("read secrets").to_owned()
  }
}

/// Reads the secrets from the DB
fn init() -> Result<Secret, Error> {
  let db_url = match get_database_url_from_env() {
    Ok(url) => url,
    Err(_) => Settings::get().get_database_url(),
  };

  let conn = PgConnection::establish(&db_url).expect("Couldn't get DB connection for Secrets.");
  read_secrets(&conn)
}

fn read_secrets(conn: &PgConnection) -> Result<Secret, Error> {
  use lemmy_db_schema::schema::secret::dsl::*;
  secret.first::<Secret>(conn)
}
