use crate::settings::structs_opt::{
  CaptchaConfigOpt,
  DatabaseConfigOpt,
  EmailConfigOpt,
  FederationConfigOpt,
  RateLimitConfigOpt,
  SettingsOpt,
  SetupOpt,
};
use std::{env, str::FromStr};

pub(in crate::settings) fn parse_from_env() -> SettingsOpt {
  SettingsOpt {
    hostname: env_var("HOSTNAME"),
    bind: env_var("BIND"),
    port: env_var("PORT"),
    tls_enabled: env_var("TLS_ENABLED"),
    jwt_secret: env_var("JWT_SECRET"),
    pictrs_url: env_var("PICTRS_URL"),
    iframely_url: env_var("IFRAMELY_URL"),
    rate_limit: Some(RateLimitConfigOpt {
      message: env_var("RATE_LIMIT__MESSAGE"),
      message_per_second: env_var("RATE_LIMIT__MESSAGE_PER_SECOND"),
      post: env_var("RATE_LIMIT__POST"),
      post_per_second: env_var("RATE_LIMIT__POST_PER_SECOND"),
      register: env_var("RATE_LIMIT__REGISTER"),
      register_per_second: env_var("RATE_LIMIT__REGISTER_PER_SECOND"),
      image: env_var("RATE_LIMIT__IMAGE"),
      image_per_second: env_var("RATE_LIMIT__IMAGE_PER_SECOND"),
    }),
    email: Some(EmailConfigOpt {
      smtp_server: env_var("EMAIL__SMTP_SERVER"),
      smtp_login: Some(env_var("EMAIL__SMTP_LOGIN")),
      smtp_password: Some(env_var("EMAIL__SMTP_PASSWORD")),
      smtp_from_address: env_var("EMAIL__SMTP_FROM_ADDRESS"),
      use_tls: env_var("EMAIL__USE_TLS"),
    }),
    federation: Some(FederationConfigOpt {
      enabled: env_var("FEDERATION__ENABLED"),
      allowed_instances: env_var("FEDERATION__ALLOWED_INSTANCES"),
      blocked_instances: env_var("FEDERATION__BLOCKED_INSTANCES"),
    }),
    captcha: Some(CaptchaConfigOpt {
      enabled: env_var("CAPTCHA__ENABLED"),
      difficulty: env_var("CAPTCHA__DIFFICULTY"),
    }),
    setup: Some(SetupOpt {
      admin_username: env_var("SETUP__ADMIN_USERNAME"),
      admin_password: env_var("SETUP__ADMIN_PASSWORD"),
      admin_email: Some(env_var("SETUP__ADMIN_EMAIL")),
      site_name: env_var("SETUP__ADMIN_SITE_NAME"),
    }),
    database: Some(DatabaseConfigOpt {
      user: env_var("DATABASE__USER"),
      password: env_var("DATABASE__PASSWORD"),
      host: env_var("DATABASE__HOST"),
      port: env_var("DATABASE__PORT"),
      database: env_var("DATABASE__DATABASE"),
      pool_size: env_var("DATABASE__POOL_SIZE"),
    }),
  }
}

fn env_var<T>(name: &str) -> Option<T>
where
  T: FromStr,
  <T as FromStr>::Err: std::fmt::Debug,
{
  // TODO: probably remove the unwrap
  env::var(format!("LEMMY_{}", name))
    .ok()
    .map(|v| T::from_str(&v).unwrap())
}
