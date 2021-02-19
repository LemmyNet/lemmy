use crate::settings::{structs::*, structs_opt::*};

pub(in crate::settings) trait Merge<T> {
  fn merge(&mut self, opt: T);
}

impl Merge<SettingsOpt> for Settings {
  fn merge(&mut self, opt: SettingsOpt) {
    overwrite_if_some(&mut self.hostname, opt.hostname);
    overwrite_if_some(&mut self.bind, opt.bind);
    overwrite_if_some(&mut self.port, opt.port);
    overwrite_if_some(&mut self.tls_enabled, opt.tls_enabled);
    overwrite_if_some(&mut self.jwt_secret, opt.jwt_secret);
    overwrite_if_some(&mut self.pictrs_url, opt.pictrs_url);
    overwrite_if_some(&mut self.iframely_url, opt.iframely_url);
    merge_if_some(&mut self.captcha, opt.captcha);
    merge_if_some(&mut self.rate_limit, opt.rate_limit);
    merge_if_some_opt(&mut self.email, opt.email);
    merge_if_some_opt(&mut self.setup, opt.setup);
    merge_if_some(&mut self.federation, opt.federation);
    merge_if_some(&mut self.database, opt.database);
  }
}

impl Merge<RateLimitConfigOpt> for RateLimitConfig {
  fn merge(&mut self, opt: RateLimitConfigOpt) {
    overwrite_if_some(&mut self.message, opt.message);
    overwrite_if_some(&mut self.message_per_second, opt.message_per_second);
    overwrite_if_some(&mut self.post, opt.post);
    overwrite_if_some(&mut self.post_per_second, opt.post_per_second);
    overwrite_if_some(&mut self.register, opt.register);
    overwrite_if_some(&mut self.register_per_second, opt.register_per_second);
    overwrite_if_some(&mut self.image, opt.image);
    overwrite_if_some(&mut self.image_per_second, opt.image_per_second);
  }
}

impl Merge<SetupOpt> for Setup {
  fn merge(&mut self, opt: SetupOpt) {
    overwrite_if_some(&mut self.admin_username, opt.admin_username);
    overwrite_if_some(&mut self.admin_password, opt.admin_password);
    overwrite_if_some(&mut self.admin_email, opt.admin_email);
    overwrite_if_some(&mut self.site_name, opt.site_name);
  }
}

impl Merge<EmailConfigOpt> for EmailConfig {
  fn merge(&mut self, opt: EmailConfigOpt) {
    overwrite_if_some(&mut self.smtp_server, opt.smtp_server);
    overwrite_if_some(&mut self.smtp_login, opt.smtp_login);
    overwrite_if_some(&mut self.smtp_password, opt.smtp_password);
    overwrite_if_some(&mut self.smtp_from_address, opt.smtp_from_address);
    overwrite_if_some(&mut self.use_tls, opt.use_tls);
  }
}

impl Merge<DatabaseConfigOpt> for DatabaseConfig {
  fn merge(&mut self, opt: DatabaseConfigOpt) {
    overwrite_if_some(&mut self.user, opt.user);
    overwrite_if_some(&mut self.password, opt.password);
    overwrite_if_some(&mut self.host, opt.host);
    overwrite_if_some(&mut self.port, opt.port);
    overwrite_if_some(&mut self.database, opt.database);
    overwrite_if_some(&mut self.pool_size, opt.pool_size);
  }
}

impl Merge<FederationConfigOpt> for FederationConfig {
  fn merge(&mut self, opt: FederationConfigOpt) {
    overwrite_if_some(&mut self.enabled, opt.enabled);
    overwrite_if_some(&mut self.allowed_instances, opt.allowed_instances);
    overwrite_if_some(&mut self.blocked_instances, opt.blocked_instances);
  }
}

impl Merge<CaptchaConfigOpt> for CaptchaConfig {
  fn merge(&mut self, opt: CaptchaConfigOpt) {
    overwrite_if_some(&mut self.enabled, opt.enabled);
    overwrite_if_some(&mut self.difficulty, opt.difficulty);
  }
}

fn overwrite_if_some<T>(lhs: &mut T, rhs: Option<T>) {
  if let Some(x) = rhs {
    *lhs = x;
  }
}

fn merge_if_some<T, U>(lhs: &mut T, rhs: Option<U>)
where
  T: Merge<U>,
{
  if let Some(x) = rhs {
    lhs.merge(x);
  }
}

fn merge_if_some_opt<T, U>(lhs: &mut Option<T>, rhs: Option<U>)
where
  T: Merge<U>,
{
  if let Some(x) = rhs {
    if let Some(y) = lhs {
      y.merge(x)
    }
  }
}
