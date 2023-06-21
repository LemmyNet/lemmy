use std::env;

pub fn version() -> String {
  env::var("LEMMY_VERSION").unwrap_or("Unknown version".into())
}
