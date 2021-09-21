use crate::{diesel::RunQueryDsl, lazy_static::__Deref};
use diesel::PgConnection;
use lemmy_db_schema::source::secrets::Secrets;
use lemmy_utils::LemmyError;
use std::sync::RwLock;

pub trait Secrets_ {
  fn read_jwt_secret(conn: &PgConnection) -> Result<String, LemmyError>;
}

// TODO: thread_local! might be better in terms of performance, but i couldnt get it to work
lazy_static! {
  static ref JWT_SECRET: RwLock<Option<String>> = RwLock::new(None);
}

impl Secrets_ for Secrets {
  fn read_jwt_secret(conn: &PgConnection) -> Result<String, LemmyError> {
    use lemmy_db_schema::schema::secrets::dsl::*;
    let jwt_option: Option<String> = JWT_SECRET.read().unwrap().deref().clone();
    match jwt_option {
      Some(j) => Ok(j),
      None => {
        let jwt = secrets.first::<Self>(conn).map(|s| s.jwt_secret)?;
        let jwt_static = JWT_SECRET.write();
        let mut jwt_static = jwt_static.unwrap();
        *jwt_static = Some(jwt.clone());
        Ok(jwt)
      }
    }
  }
}
