use actix_cors::Cors;
use lemmy_utils::settings::structs::Settings;

pub mod code_migrations;
pub mod prometheus_metrics;
pub mod scheduled_tasks;

pub fn cors_config(settings: &Settings) -> Cors {
  let self_origin = settings.get_protocol_and_hostname();
  let cors_origin_setting = settings.cors_origin();

  // A default setting for either wildcard, or None
  let cors_default = Cors::default()
    .allow_any_origin()
    .allow_any_method()
    .allow_any_header()
    .expose_any_header()
    .max_age(3600);

  match (cors_origin_setting.clone(), cfg!(debug_assertions)) {
    (Some(origin), false) => {
      // Need to call send_wildcard() explicitly, passing this into allowed_origin() results in
      // error
      if origin == "*" {
        cors_default
      } else {
        Cors::default()
          .allowed_origin(&origin)
          .allowed_origin(&self_origin)
          .allow_any_method()
          .allow_any_header()
          .expose_any_header()
          .max_age(3600)
      }
    }
    _ => cors_default,
  }
}
