use crate::settings::{structs::*, structs_opt::*};

pub(in crate::settings) trait Merge<T> {
  fn merge(self, opt: T) -> Self;
}

impl Merge<SettingsOpt> for Settings {
  fn merge(self, opt: SettingsOpt) -> Self {
    Settings {
      setup: merge_structs(self.setup, opt.setup),
      database: merge_structs(self.database, opt.database),
      hostname: opt.hostname.unwrap_or(self.hostname),
      bind: opt.bind.unwrap_or(self.bind),
      port: opt.port.unwrap_or(self.port),
      tls_enabled: opt.tls_enabled.unwrap_or(self.tls_enabled),
      jwt_secret: opt.jwt_secret.unwrap_or(self.jwt_secret),
      pictrs_url: opt.pictrs_url.unwrap_or(self.pictrs_url),
      iframely_url: opt.iframely_url.unwrap_or(self.iframely_url),
      rate_limit: merge_structs(self.rate_limit, opt.rate_limit),
      email: merge_structs(self.email, opt.email),
      federation: merge_structs(self.federation, opt.federation),
      captcha: merge_structs(self.captcha, opt.captcha),
    }
  }
}

impl Merge<RateLimitConfigOpt> for RateLimitConfig {
  fn merge(self, opt: RateLimitConfigOpt) -> Self {
    RateLimitConfig {
      message: opt.message.unwrap_or(self.message),
      message_per_second: opt.message_per_second.unwrap_or(self.message_per_second),
      post: opt.post.unwrap_or(self.post),
      post_per_second: opt.post_per_second.unwrap_or(self.post_per_second),
      register: opt.register.unwrap_or(self.register),
      register_per_second: opt.register_per_second.unwrap_or(self.register_per_second),
      image: opt.image.unwrap_or(self.image),
      image_per_second: opt.image_per_second.unwrap_or(self.image_per_second),
    }
  }
}

impl Merge<SetupOpt> for Option<Setup> {
  fn merge(self, opt: SetupOpt) -> Self {
    if let Some(setup) = self {
      Some(Setup {
        admin_username: opt.admin_username.unwrap_or(setup.admin_username),
        admin_password: opt.admin_password.unwrap_or(setup.admin_password),
        admin_email: opt.admin_email.unwrap_or(setup.admin_email),
        site_name: opt.site_name.unwrap_or(setup.site_name),
      })
    } else if let (Some(admin_username), Some(admin_password), Some(site_name)) =
      (opt.admin_username, opt.admin_password, opt.site_name)
    {
      Some(Setup {
        admin_username,
        admin_password,
        admin_email: opt.admin_email.flatten(),
        site_name,
      })
    } else {
      None
    }
  }
}

impl Merge<EmailConfigOpt> for Option<EmailConfig> {
  fn merge(self, opt: EmailConfigOpt) -> Self {
    if let Some(email_config) = self {
      Some(EmailConfig {
        smtp_server: opt.smtp_server.unwrap_or(email_config.smtp_server),
        smtp_login: opt.smtp_login.unwrap_or(email_config.smtp_login),
        smtp_password: opt.smtp_password.unwrap_or(email_config.smtp_password),
        smtp_from_address: opt
          .smtp_from_address
          .unwrap_or(email_config.smtp_from_address),
        use_tls: opt.use_tls.unwrap_or(email_config.use_tls),
      })
    } else if let (Some(smtp_server), Some(smtp_from_address), Some(use_tls)) =
      (opt.smtp_server, opt.smtp_from_address, opt.use_tls)
    {
      Some(EmailConfig {
        smtp_server,
        smtp_login: opt
          .smtp_login
          .or(self.clone().map(|s| s.smtp_login))
          .flatten(),
        smtp_password: opt
          .smtp_password
          .or(self.map(|s| s.smtp_password))
          .flatten(),
        smtp_from_address,
        use_tls,
      })
    } else {
      None
    }
  }
}

impl Merge<DatabaseConfigOpt> for DatabaseConfig {
  fn merge(self, opt: DatabaseConfigOpt) -> Self {
    DatabaseConfig {
      user: opt.user.unwrap_or(self.user),
      password: opt.password.unwrap_or(self.password),
      host: opt.host.unwrap_or(self.host),
      port: opt.port.unwrap_or(self.port),
      database: opt.database.unwrap_or(self.database),
      pool_size: opt.pool_size.unwrap_or(self.pool_size),
    }
  }
}

impl Merge<FederationConfigOpt> for FederationConfig {
  fn merge(self, opt: FederationConfigOpt) -> Self {
    FederationConfig {
      enabled: opt.enabled.unwrap_or(self.enabled),
      allowed_instances: opt.allowed_instances.unwrap_or(self.allowed_instances),
      blocked_instances: opt.blocked_instances.unwrap_or(self.blocked_instances),
    }
  }
}

impl Merge<CaptchaConfigOpt> for CaptchaConfig {
  fn merge(self, opt: CaptchaConfigOpt) -> Self {
    CaptchaConfig {
      enabled: opt.enabled.unwrap_or(self.enabled),
      difficulty: opt.difficulty.unwrap_or(self.difficulty),
    }
  }
}

fn merge_structs<T, U>(lhs: T, rhs: Option<U>) -> T
where
  T: Merge<U> + std::clone::Clone,
{
  if let Some(x) = rhs {
    lhs.merge(x)
  } else {
    lhs.to_owned()
  }
}
