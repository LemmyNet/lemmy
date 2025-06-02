use actix_cors::Cors;
use lemmy_utils::settings::structs::Settings;

pub mod prometheus_metrics;
pub mod scheduled_tasks;
pub mod setup_local_site;

pub fn cors_config(settings: &Settings) -> Cors {
  let self_origin = settings.get_protocol_and_hostname();
  let cors_origin_setting = settings.cors_origin();

  let mut cors = Cors::default()
    .allow_any_method()
    .allow_any_header()
    .expose_any_header()
    .max_age(3600);

  if cfg!(debug_assertions)
    || cors_origin_setting.is_empty()
    || cors_origin_setting.contains(&"*".to_string())
  {
    cors = cors.allow_any_origin();
  } else {
    cors = cors.allowed_origin(&self_origin);
    for c in cors_origin_setting {
      cors = cors.allowed_origin(&c);
    }
  }
  cors
}
